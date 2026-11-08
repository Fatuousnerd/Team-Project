use crate::error::AppResult;
use crate::market::MockMarket;
use crate::types::*;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

/// A single-symbol order book with FIFO price-level matching.
///
/// Supports Market and Limit orders. Stop/StopLimit are intentionally rejected
/// until a trigger scheduler is added later.
///
/// Matching avoids holding simultaneous mutable borrows of `self` by taking an
/// order out of a level, mutating it locally, then putting it back (or
/// discarding it) before touching `self` again.
pub struct OrderBook {
    pub symbol: Symbol,
    /// BTreeMap is ascending by price. Bids are read from the back (highest).
    pub bids: BTreeMap<Price, Vec<Order>>,
    pub asks: BTreeMap<Price, Vec<Order>>,
    /// Source of truth for order lifecycle (resting and executed).
    pub orders: HashMap<Uuid, Order>,
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(b), Some(a)) => Some((b + a) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Remove the first order at the best price level.
    /// `top = true` → highest price (best bid); `top = false` → lowest price (best ask).
    fn pop_best(levels: &mut BTreeMap<Price, Vec<Order>>, top: bool) -> Option<(Price, Order)> {
        let key = if top {
            levels.keys().next_back().copied()?
        } else {
            levels.keys().next().copied()?
        };
        let order = levels.get_mut(&key).unwrap().remove(0);
        if levels.get(&key).map_or(false, |l| l.is_empty()) {
            levels.remove(&key);
        }
        Some((key, order))
    }

    /// Match an incoming order against the book, resting any remainder (limit
    /// orders) back into the book. Returns the trades produced.
    pub fn place_order(&mut self, mut order: Order) -> AppResult<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut remaining = order.quantity - order.filled_quantity;
        if remaining <= Decimal::ZERO {
            return Ok(trades);
        }

        match order.order_type {
            OrderType::Market => {
                match order.side {
                    OrderSide::Buy => {
                        while remaining > Decimal::ZERO {
                            let Some((ask_price, mut book_order)) =
                                Self::pop_best(&mut self.asks, false)
                            else {
                                break;
                            };
                            let available = book_order.quantity - book_order.filled_quantity;
                            if available <= Decimal::ZERO {
                                continue;
                            }
                            let trade_qty = remaining.min(available);
                            let trade = Trade {
                                id: Uuid::new_v4(),
                                order_id: order.id,
                                symbol: self.symbol.clone(),
                                side: order.side,
                                price: ask_price,
                                quantity: trade_qty,
                                executed_at: Utc::now(),
                            };
                            trades.push(trade.clone());
                            book_order.filled_quantity += trade_qty;
                            remaining -= trade_qty;
                            order.filled_quantity += trade_qty;
                            book_order.status = if book_order.filled_quantity >= book_order.quantity {
                                OrderStatus::Filled
                            } else {
                                OrderStatus::PartiallyFilled
                            };
                            book_order.updated_at = Utc::now();
                            if book_order.filled_quantity < book_order.quantity {
                                self.asks
                                    .entry(ask_price)
                                    .or_default()
                                    .insert(0, book_order.clone());
                            }
                            if let Some(o) = self.orders.get_mut(&book_order.id) {
                                *o = book_order.clone();
                            }
                        }
                    }
                    OrderSide::Sell => {
                        while remaining > Decimal::ZERO {
                            let Some((bid_price, mut book_order)) =
                                Self::pop_best(&mut self.bids, true)
                            else {
                                break;
                            };
                            let available = book_order.quantity - book_order.filled_quantity;
                            if available <= Decimal::ZERO {
                                continue;
                            }
                            let trade_qty = remaining.min(available);
                            let trade = Trade {
                                id: Uuid::new_v4(),
                                order_id: order.id,
                                symbol: self.symbol.clone(),
                                side: order.side,
                                price: bid_price,
                                quantity: trade_qty,
                                executed_at: Utc::now(),
                            };
                            trades.push(trade.clone());
                            book_order.filled_quantity += trade_qty;
                            remaining -= trade_qty;
                            order.filled_quantity += trade_qty;
                            book_order.status = if book_order.filled_quantity >= book_order.quantity {
                                OrderStatus::Filled
                            } else {
                                OrderStatus::PartiallyFilled
                            };
                            book_order.updated_at = Utc::now();
                            if book_order.filled_quantity < book_order.quantity {
                                self.bids
                                    .entry(bid_price)
                                    .or_default()
                                    .insert(0, book_order.clone());
                            }
                            if let Some(o) = self.orders.get_mut(&book_order.id) {
                                *o = book_order.clone();
                            }
                        }
                    }
                }
            }
            OrderType::Limit => {
                let limit_price = order
                    .price
                    .ok_or_else(|| crate::error::AppError::BadRequest("Limit order requires a price".to_string()))?;
                match order.side {
                    OrderSide::Buy => {
                        while remaining > Decimal::ZERO {
                            let Some(&ask) = self.asks.keys().next() else { break };
                            if ask > limit_price {
                                break;
                            }
                            let Some((ask_price, mut book_order)) =
                                Self::pop_best(&mut self.asks, false)
                            else {
                                break;
                            };
                            let available = book_order.quantity - book_order.filled_quantity;
                            if available <= Decimal::ZERO {
                                continue;
                            }
                            let trade_qty = remaining.min(available);
                            let trade = Trade {
                                id: Uuid::new_v4(),
                                order_id: order.id,
                                symbol: self.symbol.clone(),
                                side: order.side,
                                price: ask_price,
                                quantity: trade_qty,
                                executed_at: Utc::now(),
                            };
                            trades.push(trade.clone());
                            book_order.filled_quantity += trade_qty;
                            remaining -= trade_qty;
                            order.filled_quantity += trade_qty;
                            book_order.status = if book_order.filled_quantity >= book_order.quantity {
                                OrderStatus::Filled
                            } else {
                                OrderStatus::PartiallyFilled
                            };
                            book_order.updated_at = Utc::now();
                            if book_order.filled_quantity < book_order.quantity {
                                self.asks
                                    .entry(ask_price)
                                    .or_default()
                                    .insert(0, book_order.clone());
                            }
                            if let Some(o) = self.orders.get_mut(&book_order.id) {
                                *o = book_order.clone();
                            }
                        }
                        if remaining > Decimal::ZERO {
                            order.filled_quantity = order.quantity - remaining;
                            self.bids.entry(limit_price).or_default().push(order.clone());
                        }
                    }
                    OrderSide::Sell => {
                        while remaining > Decimal::ZERO {
                            let Some(&bid) = self.bids.keys().next_back() else { break };
                            if bid < limit_price {
                                break;
                            }
                            let Some((bid_price, mut book_order)) =
                                Self::pop_best(&mut self.bids, true)
                            else {
                                break;
                            };
                            let available = book_order.quantity - book_order.filled_quantity;
                            if available <= Decimal::ZERO {
                                continue;
                            }
                            let trade_qty = remaining.min(available);
                            let trade = Trade {
                                id: Uuid::new_v4(),
                                order_id: order.id,
                                symbol: self.symbol.clone(),
                                side: order.side,
                                price: bid_price,
                                quantity: trade_qty,
                                executed_at: Utc::now(),
                            };
                            trades.push(trade.clone());
                            book_order.filled_quantity += trade_qty;
                            remaining -= trade_qty;
                            order.filled_quantity += trade_qty;
                            book_order.status = if book_order.filled_quantity >= book_order.quantity {
                                OrderStatus::Filled
                            } else {
                                OrderStatus::PartiallyFilled
                            };
                            book_order.updated_at = Utc::now();
                            if book_order.filled_quantity < book_order.quantity {
                                self.bids
                                    .entry(bid_price)
                                    .or_default()
                                    .insert(0, book_order.clone());
                            }
                            if let Some(o) = self.orders.get_mut(&book_order.id) {
                                *o = book_order.clone();
                            }
                        }
                        if remaining > Decimal::ZERO {
                            order.filled_quantity = order.quantity - remaining;
                            self.asks.entry(limit_price).or_default().push(order.clone());
                        }
                    }
                }
            }
            OrderType::Stop | OrderType::StopLimit => {
                return Err(crate::error::AppError::BadRequest(
                    "Stop orders are not implemented yet".to_string(),
                ));
            }
        }

        order.status = if order.filled_quantity >= order.quantity {
            OrderStatus::Filled
        } else if order.filled_quantity > Decimal::ZERO {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Open
        };
        order.updated_at = Utc::now();
        self.orders.insert(order.id, order.clone());

        Ok(trades)
    }

    pub fn cancel_order(&mut self, id: &Uuid) -> AppResult<()> {
        let order = self
            .orders
            .get(id)
            .cloned()
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Order {id} not found")))?;
        if matches!(order.status, OrderStatus::Filled | OrderStatus::Cancelled) {
            return Err(crate::error::AppError::BadRequest(
                "Cannot cancel filled or cancelled order".to_string(),
            ));
        }
        if let Some(price) = order.price {
            match order.side {
                OrderSide::Buy => {
                    if let Some(level) = self.bids.get_mut(&price) {
                        level.retain(|o| o.id != *id);
                        if level.is_empty() {
                            self.bids.remove(&price);
                        }
                    }
                }
                OrderSide::Sell => {
                    if let Some(level) = self.asks.get_mut(&price) {
                        level.retain(|o| o.id != *id);
                        if level.is_empty() {
                            self.asks.remove(&price);
                        }
                    }
                }
            }
        }
        let mut order = order;
        order.status = OrderStatus::Cancelled;
        order.updated_at = Utc::now();
        self.orders.insert(*id, order);
        Ok(())
    }

    pub fn get_order(&self, id: &Uuid) -> Option<&Order> {
        self.orders.get(id)
    }

    pub fn list_orders(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }
}

/// Top-level trading engine: market data, order books, accounts, and trade log.
pub struct TradingEngine {
    pub market: Arc<RwLock<MockMarket>>,
    pub order_books: Arc<RwLock<HashMap<Symbol, OrderBook>>>,
    pub accounts: Arc<RwLock<HashMap<Uuid, Account>>>,
    pub trades: Arc<Mutex<Vec<Trade>>>,
}

impl TradingEngine {
    pub fn new() -> Self {
        Self {
            market: Arc::new(RwLock::new(MockMarket::new())),
            order_books: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn seed_accounts(&self) {
        let mut accounts = self.accounts.write().unwrap();
        if !accounts.is_empty() {
            return;
        }
        let balances = [
            rust_decimal::Decimal::from(100_000),
            rust_decimal::Decimal::from(250_000),
        ];
        for (i, balance) in balances.into_iter().enumerate() {
            let id = Uuid::new_v4();
            accounts.insert(
                id,
                Account {
                    id,
                    name: format!("Account {}", i + 1),
                    balance,
                    equity: balance,
                    margin_used: rust_decimal::Decimal::ZERO,
                    positions: Vec::new(),
                },
            );
        }
    }

    pub fn place_order(&self, req: PlaceOrderRequest) -> AppResult<Order> {
        let account_id = req.account_id;
        let symbol = Symbol::new(req.symbol);

        {
            let accounts = self.accounts.read().unwrap();
            if !accounts.contains_key(&account_id) {
                return Err(crate::error::AppError::NotFound(format!(
                    "Account {account_id} not found"
                )));
            }
        }

        let order = Order {
            id: Uuid::new_v4(),
            account_id,
            symbol: symbol.clone(),
            side: req.side,
            order_type: req.r#type,
            price: req.price,
            quantity: req.quantity,
            filled_quantity: rust_decimal::Decimal::ZERO,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let trades = {
            let mut books = self.order_books.write().unwrap();
            let book = books
                .entry(symbol.clone())
                .or_insert_with(|| OrderBook::new(symbol.clone()));
            book.place_order(order.clone())?
        };

        {
            let mut trades_log = self.trades.lock().unwrap();
            trades_log.extend(trades.clone());
        }

        self.update_account_after_fill(account_id, &symbol, &trades)?;

        let final_order = {
            let books = self.order_books.read().unwrap();
            let book = books
                .get(&symbol)
                .ok_or_else(|| crate::error::AppError::Internal("order book missing".to_string()))?;
            book.get_order(&order.id)
                .cloned()
                .ok_or_else(|| crate::error::AppError::Internal("order missing after match".to_string()))?
        };

        Ok(final_order)
    }

    pub fn cancel_order(&self, id: &Uuid) -> AppResult<Order> {
        let symbol = {
            let books = self.order_books.read().unwrap();
            books
                .iter()
                .find_map(|(sym, book)| book.get_order(id).map(|_| sym.clone()))
                .ok_or_else(|| crate::error::AppError::NotFound(format!("Order {id} not found")))?
        };

        let mut books = self.order_books.write().unwrap();
        let book = books
            .get_mut(&symbol)
            .ok_or_else(|| crate::error::AppError::Internal("order book missing".to_string()))?;
        book.cancel_order(id)?;
        let cancelled = book
            .get_order(id)
            .cloned()
            .ok_or_else(|| crate::error::AppError::Internal("order vanished".to_string()))?;
        Ok(cancelled)
    }

    pub fn get_order(&self, id: &Uuid) -> AppResult<Order> {
        let books = self.order_books.read().unwrap();
        for book in books.values() {
            if let Some(order) = book.get_order(id) {
                return Ok(order.clone());
            }
        }
        Err(crate::error::AppError::NotFound(format!(
            "Order {id} not found"
        )))
    }

    pub fn list_orders(&self) -> Vec<Order> {
        let books = self.order_books.read().unwrap();
        books
            .values()
            .flat_map(|b| b.list_orders())
            .cloned()
            .collect()
    }

    pub fn get_account(&self, id: Uuid) -> AppResult<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts
            .get(&id)
            .cloned()
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Account {id} not found")))
    }

    pub fn list_accounts(&self) -> Vec<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts.values().cloned().collect()
    }

    fn update_account_after_fill(
        &self,
        account_id: Uuid,
        symbol: &Symbol,
        trades: &[Trade],
    ) -> AppResult<()> {
        if trades.is_empty() {
            return Ok(());
        }

        let mut accounts = self.accounts.write().unwrap();
        let account = accounts
            .get_mut(&account_id)
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Account {account_id} not found")))?;

        for trade in trades {
            let notional = trade.price * trade.quantity;
            match trade.side {
                OrderSide::Buy => {
                    account.balance -= notional;
                    if let Some(pos) = account
                        .positions
                        .iter_mut()
                        .find(|p| p.symbol == *symbol)
                    {
                        let total_qty = pos.quantity + trade.quantity;
                        pos.average_price = ((pos.average_price * pos.quantity) + notional) / total_qty;
                        pos.quantity = total_qty;
                    } else {
                        account.positions.push(Position {
                            symbol: symbol.clone(),
                            quantity: trade.quantity,
                            average_price: trade.price,
                            unrealized_pnl: rust_decimal::Decimal::ZERO,
                        });
                    }
                }
                OrderSide::Sell => {
                    account.balance += notional;
                    if let Some(pos) = account
                        .positions
                        .iter_mut()
                        .find(|p| p.symbol == *symbol)
                    {
                        let total_qty = pos.quantity - trade.quantity;
                        if total_qty == rust_decimal::Decimal::ZERO {
                            pos.quantity = rust_decimal::Decimal::ZERO;
                            pos.average_price = rust_decimal::Decimal::ZERO;
                        } else {
                            pos.quantity = total_qty;
                        }
                    }
                }
            }
        }

        // Mark-to-market unrealized PnL against the latest mock price.
        let market = self.market.read().unwrap();
        if let Some(md) = market.get(symbol) {
            for pos in &mut account.positions {
                if pos.symbol == *symbol {
                    pos.unrealized_pnl = (md.last_price - pos.average_price) * pos.quantity;
                }
            }
        }

        let position_value: rust_decimal::Decimal = account
            .positions
            .iter()
            .map(|p| p.quantity * p.average_price)
            .sum();
        account.equity = account.balance + position_value;
        account.margin_used = rust_decimal::Decimal::ZERO;

        Ok(())
    }
}

impl Default for TradingEngine {
    fn default() -> Self {
        Self::new()
    }
}
