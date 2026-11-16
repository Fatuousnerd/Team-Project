import {
	Bot,
	LayoutDashboard,
	LineChart,
	ListOrdered,
	Settings,
	Wallet,
} from "lucide-react";
import { Link, useLocation } from "react-router";
import { Brand } from "@/components/app/brand";
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarGroupLabel,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuBadge,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarSeparator,
} from "@/components/ui/sidebar";
import { useMarketStore } from "@/lib/mock/market-store";

const NAV_GROUPS = [
	{
		label: "Trading",
		items: [
			{ label: "Dashboard", to: "/", icon: LayoutDashboard },
			{ label: "Markets", to: "/markets", icon: LineChart },
			{ label: "Portfolio", to: "/portfolio", icon: Wallet },
			{ label: "Orders", to: "/orders", icon: ListOrdered },
		],
	},
	{
		label: "Automation",
		items: [{ label: "Strategies", to: "/strategies", icon: Bot }],
	},
	{
		label: "System",
		items: [{ label: "Settings", to: "/settings", icon: Settings }],
	},
];

export function AppSidebar() {
	const { pathname } = useLocation();
	const latency = useMarketStore((state) => state.latency);
	const orders = useMarketStore((state) => state.orders);
	const openOrders = orders.filter((order) => order.status === "Open").length;
	const running = useMarketStore(
		(state) => state.strategies.filter((s) => s.status === "running").length,
	);

	const isActive = (to: string) =>
		to === "/" ? pathname === "/" : pathname.startsWith(to);

	return (
		<Sidebar collapsible="icon">
			<SidebarHeader className="h-14 justify-center px-2 py-0">
				<Link
					to="/"
					className="flex items-center rounded-md px-0.5 outline-none focus-visible:ring-2 focus-visible:ring-ring/50"
				>
					<Brand />
				</Link>
			</SidebarHeader>

			<SidebarSeparator />

			<SidebarContent>
				{NAV_GROUPS.map((group) => (
					<SidebarGroup key={group.label}>
						<SidebarGroupLabel>{group.label}</SidebarGroupLabel>
						<SidebarMenu>
							{group.items.map((item) => (
								<SidebarMenuItem key={item.to}>
									<SidebarMenuButton
										asChild
										isActive={isActive(item.to)}
										tooltip={item.label}
									>
										<Link to={item.to}>
											<item.icon />
											<span>{item.label}</span>
										</Link>
									</SidebarMenuButton>
									{item.label === "Orders" && openOrders > 0 ? (
										<SidebarMenuBadge className="num">
											{openOrders}
										</SidebarMenuBadge>
									) : null}
									{item.label === "Strategies" && running > 0 ? (
										<SidebarMenuBadge className="num text-primary">
											{running}
										</SidebarMenuBadge>
									) : null}
								</SidebarMenuItem>
							))}
						</SidebarMenu>
					</SidebarGroup>
				))}
			</SidebarContent>

			<SidebarFooter className="gap-2">
				<div className="rounded-lg border border-border/80 bg-card p-2.5 group-data-[collapsible=icon]:hidden">
					<div className="flex items-center justify-between">
						<span className="text-xs font-medium">Paper engine</span>
						<span className="inline-flex items-center gap-1.5 text-[10px] font-medium text-primary">
							<span className="relative flex size-1.5">
								<span className="absolute inline-flex size-full animate-ping rounded-full bg-primary opacity-60" />
								<span className="relative inline-flex size-1.5 rounded-full bg-primary" />
							</span>
							Live
						</span>
					</div>
					<div className="mt-2 flex flex-col gap-1 text-[11px] text-muted-foreground">
						<div className="flex items-center justify-between">
							<span>Feed</span>
							<span className="num">mock · {latency} ms</span>
						</div>
						<div className="flex items-center justify-between">
							<span>Symbols</span>
							<span className="num">19</span>
						</div>
					</div>
				</div>

				<div className="flex items-center gap-2 rounded-lg px-1 py-1 group-data-[collapsible=icon]:justify-center">
					<div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-secondary text-xs font-semibold text-secondary-foreground">
						FT
					</div>
					<div className="flex min-w-0 flex-col leading-tight group-data-[collapsible=icon]:hidden">
						<span className="truncate text-xs font-medium">Fatuous</span>
						<span className="truncate text-[10px] text-muted-foreground">
							Paper trader
						</span>
					</div>
				</div>
			</SidebarFooter>
		</Sidebar>
	);
}
