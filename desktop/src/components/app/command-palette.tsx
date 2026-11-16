import {
	Bot,
	CircleCheck,
	Info,
	LayoutDashboard,
	LineChart,
	ListOrdered,
	Settings,
	TriangleAlert,
	Wallet,
	Workflow,
} from "lucide-react";
import { useNavigate } from "react-router";
import {
	Command,
	CommandDialog,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
	CommandSeparator,
} from "@/components/ui/command";
import { fmtPrice } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import type { ActivityKind } from "@/lib/mock/types";

const PAGES = [
	{ label: "Dashboard", to: "/", icon: LayoutDashboard },
	{ label: "Markets", to: "/markets", icon: LineChart },
	{ label: "Portfolio", to: "/portfolio", icon: Wallet },
	{ label: "Orders", to: "/orders", icon: ListOrdered },
	{ label: "Strategies", to: "/strategies", icon: Bot },
	{ label: "Settings", to: "/settings", icon: Settings },
];

export const ACTIVITY_ICONS: Record<ActivityKind, typeof Info> = {
	fill: CircleCheck,
	bot: Workflow,
	alert: TriangleAlert,
	system: Info,
};

export function CommandPalette({
	open,
	onOpenChange,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}) {
	const navigate = useNavigate();
	const quotes = useMarketStore((state) => state.quotes);

	const go = (to: string) => {
		onOpenChange(false);
		navigate(to);
	};

	return (
		<CommandDialog
			open={open}
			onOpenChange={onOpenChange}
			title="Command palette"
			description="Jump to a page or search a symbol"
		>
			<Command>
				<CommandInput placeholder="Search pages and symbols…" />
				<CommandList>
					<CommandEmpty>No matches found.</CommandEmpty>
					<CommandGroup heading="Pages">
						{PAGES.map((page) => (
							<CommandItem
								key={page.to}
								value={`${page.label} ${page.to}`}
								onSelect={() => go(page.to)}
							>
								<page.icon className="size-4 text-muted-foreground" />
								{page.label}
							</CommandItem>
						))}
					</CommandGroup>
					<CommandSeparator />
					<CommandGroup heading="Symbols">
						{Object.values(quotes).map((quote) => (
							<CommandItem
								key={quote.symbol}
								value={`${quote.symbol} ${quote.name}`}
								onSelect={() => go(`/markets/${quote.symbol}`)}
							>
								<span className="num font-medium">{quote.symbol}</span>
								<span className="truncate text-xs text-muted-foreground">
									{quote.name}
								</span>
								<span className="num ms-auto text-xs text-muted-foreground">
									{fmtPrice(quote.price, quote.market)}
								</span>
							</CommandItem>
						))}
					</CommandGroup>
				</CommandList>
			</Command>
		</CommandDialog>
	);
}
