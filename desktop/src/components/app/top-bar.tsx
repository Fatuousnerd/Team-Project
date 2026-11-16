import { Bell, Search } from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router";
import { AccountSwitcher } from "@/components/app/account-switcher";
import {
	ACTIVITY_ICONS,
	CommandPalette,
} from "@/components/app/command-palette";
import { ConnectionPill } from "@/components/app/connection-pill";
import { ThemeToggle } from "@/components/app/theme-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { marketStatus } from "@/lib/mock/derive";
import { fmtAgo } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";

export function TopBar() {
	const [paletteOpen, setPaletteOpen] = useState(false);
	const activity = useMarketStore((state) => state.activity);
	const navigate = useNavigate();
	const isOpen = marketStatus() === "Markets open";

	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
				event.preventDefault();
				setPaletteOpen((open) => !open);
			}
		};
		window.addEventListener("keydown", onKeyDown);
		return () => window.removeEventListener("keydown", onKeyDown);
	}, []);

	return (
		<header className="sticky top-0 z-30 flex h-14 shrink-0 items-center gap-2 border-b bg-background/85 px-3 backdrop-blur-md sm:px-4">
			<SidebarTrigger className="-ms-1" />
			<Separator orientation="vertical" className="me-1 h-5!" />

			<button
				type="button"
				onClick={() => setPaletteOpen(true)}
				className="hidden h-8 w-56 items-center gap-2 rounded-lg border border-border/80 px-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted/60 lg:flex xl:w-64"
			>
				<Search className="size-3.5" />
				<span>Search symbols…</span>
				<kbd className="ms-auto rounded border border-border px-1 font-mono text-[10px] text-muted-foreground/80">
					⌘K
				</kbd>
			</button>

			<div className="ms-auto flex items-center gap-1.5">
				<Badge
					variant="outline"
					className="hidden h-6 gap-1.5 border-border/80 px-2 text-[11px] font-normal lg:inline-flex"
				>
					<span
						className={
							isOpen
								? "size-1.5 rounded-full bg-primary"
								: "size-1.5 rounded-full bg-muted-foreground/50"
						}
					/>
					{marketStatus()}
				</Badge>

				<ConnectionPill className="hidden md:inline-flex" />

				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<Button
							variant="ghost"
							size="icon"
							className="relative"
							aria-label="Notifications"
						>
							<Bell className="size-4" />
							<span className="absolute end-2 top-2 size-1.5 rounded-full bg-primary" />
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end" className="w-80 p-0">
						<div className="flex items-center justify-between px-3 py-2.5">
							<DropdownMenuLabel className="p-0">Activity</DropdownMenuLabel>
							<span className="text-[11px] text-muted-foreground">
								{activity.length} events
							</span>
						</div>
						<DropdownMenuSeparator className="my-0" />
						{activity.slice(0, 5).map((item) => {
							const Icon = ACTIVITY_ICONS[item.kind];
							return (
								<DropdownMenuItem
									key={item.id}
									className="items-start gap-2.5 px-3 py-2.5"
									onSelect={() => navigate("/orders")}
								>
									<Icon className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
									<div className="flex min-w-0 flex-col gap-0.5">
										<span className="truncate text-xs font-medium">
											{item.title}
										</span>
										<span className="truncate text-[11px] text-muted-foreground">
											{item.detail} · {fmtAgo(item.at)}
										</span>
									</div>
								</DropdownMenuItem>
							);
						})}
						<DropdownMenuSeparator className="my-0" />
						<div className="p-1">
							<DropdownMenuItem onSelect={() => navigate("/orders")}>
								<span className="mx-auto text-xs text-muted-foreground">
									View all orders
								</span>
							</DropdownMenuItem>
						</div>
					</DropdownMenuContent>
				</DropdownMenu>

				<ThemeToggle />
				<Separator orientation="vertical" className="mx-1 h-5!" />
				<div className="hidden sm:block">
					<AccountSwitcher />
				</div>
			</div>

			<CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
		</header>
	);
}
