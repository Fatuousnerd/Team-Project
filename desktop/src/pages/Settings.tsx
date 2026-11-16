import { Check, ExternalLink, Monitor, Moon, Sun } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Page } from "@/components/app/page";
import { PageHeader, SectionHeader } from "@/components/app/primitives";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { ThemeMode } from "@/config/Types/app.types";
import { useTheme } from "@/hooks/useTheme";
import { cn } from "@/lib/utils";

function SettingRow({
	title,
	description,
	children,
}: {
	title: string;
	description: string;
	children: React.ReactNode;
}) {
	return (
		<div className="flex flex-col gap-3 py-4 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between sm:gap-6">
			<div className="flex max-w-md flex-col gap-0.5">
				<span className="text-xs font-medium">{title}</span>
				<span className="text-[11px] leading-relaxed text-muted-foreground">
					{description}
				</span>
			</div>
			<div className="flex shrink-0 items-center gap-2">{children}</div>
		</div>
	);
}

const THEMES: Array<{
	value: ThemeMode;
	label: string;
	description: string;
	icon: typeof Sun;
	swatch: string;
}> = [
	{
		value: "dark",
		label: "Dark",
		description: "Ink surfaces, emerald accent",
		icon: Moon,
		swatch: "#101614",
	},
	{
		value: "light",
		label: "Light",
		description: "Paper surfaces, same accent",
		icon: Sun,
		swatch: "#f8f8f6",
	},
];

export default function Settings() {
	const { theme, setTheme } = useTheme();
	const [orderType, setOrderType] = useState("Limit");
	const [defaultQty, setDefaultQty] = useState("10");
	const [confirmOrders, setConfirmOrders] = useState(true);
	const [reduceOnly, setReduceOnly] = useState(false);
	const [notifyFills, setNotifyFills] = useState(true);
	const [notifyAlerts, setNotifyAlerts] = useState(true);
	const [notifyBots, setNotifyBots] = useState(false);

	return (
		<Page className="max-w-4xl">
			<PageHeader
				title="Settings"
				description="Terminal preferences for this desktop install. Everything stays local."
			/>

			<Card>
				<CardHeader>
					<SectionHeader
						title="Appearance"
						description="Dark first, with a matching light palette."
					/>
				</CardHeader>
				<CardContent className="grid gap-2 sm:grid-cols-2">
					{THEMES.map((option) => {
						const Icon = option.icon;
						const active = theme === option.value;
						return (
							<button
								key={option.value}
								type="button"
								onClick={() => setTheme(option.value)}
								className={cn(
									"flex items-center gap-3 rounded-lg border p-3 text-left transition-colors",
									active
										? "border-primary/40 bg-primary/5"
										: "border-border/70 hover:bg-muted/40",
								)}
							>
								<span
									className="flex size-10 shrink-0 items-center justify-center rounded-md border border-border/70"
									style={{ backgroundColor: option.swatch }}
								>
									<Icon
										className={cn(
											"size-4",
											option.value === "dark"
												? "text-white/70"
												: "text-black/50",
										)}
									/>
								</span>
								<span className="flex flex-col gap-0.5">
									<span className="text-xs font-medium">{option.label}</span>
									<span className="text-[11px] text-muted-foreground">
										{option.description}
									</span>
								</span>
								{active ? (
									<Check className="ms-auto size-4 text-primary" />
								) : null}
							</button>
						);
					})}
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<SectionHeader
						title="Trading defaults"
						description="Pre-fill values used by the order ticket."
					/>
				</CardHeader>
				<CardContent className="divide-y divide-border/60">
					<SettingRow
						title="Default order type"
						description="Ticket opens with this execution type selected."
					>
						<Select value={orderType} onValueChange={setOrderType}>
							<SelectTrigger size="sm" className="w-32">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{["Market", "Limit", "Stop"].map((option) => (
									<SelectItem key={option} value={option}>
										{option}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</SettingRow>

					<SettingRow
						title="Default quantity"
						description="Starting size for new tickets. You can override it per order."
					>
						<Input
							value={defaultQty}
							onChange={(event) => setDefaultQty(event.target.value)}
							inputMode="decimal"
							className="num w-24 text-end"
						/>
					</SettingRow>

					<SettingRow
						title="Confirm before submitting"
						description="Show a review prompt before a paper order is sent."
					>
						<Switch
							checked={confirmOrders}
							onCheckedChange={setConfirmOrders}
						/>
					</SettingRow>

					<SettingRow
						title="Reduce-only by default"
						description="Prevent accidental adds on positions you intend to trim."
					>
						<Switch checked={reduceOnly} onCheckedChange={setReduceOnly} />
					</SettingRow>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<SectionHeader
						title="Data & connectivity"
						description="Where quotes come from in this build."
					/>
				</CardHeader>
				<CardContent className="divide-y divide-border/60">
					<SettingRow
						title="Feed mode"
						description="UI is running on seeded fixtures that tick locally."
					>
						<Badge
							variant="outline"
							className="border-primary/30 bg-primary/10 text-primary"
						>
							Mock data
						</Badge>
					</SettingRow>

					<SettingRow
						title="Rust API endpoint"
						description="Target for the Axum service in crates/bot once wired."
					>
						<Input
							readOnly
							value="http://localhost:3001"
							className="num w-52 text-xs"
						/>
						<Button
							variant="outline"
							size="sm"
							onClick={() =>
								toast.success("Mock feed healthy", {
									description:
										"Local fixtures are streaming. API wiring is next.",
								})
							}
						>
							Test
						</Button>
					</SettingRow>

					<SettingRow
						title="Account mode"
						description="All orders are paper trades and never reach a broker."
					>
						<Badge variant="secondary">Paper</Badge>
					</SettingRow>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<SectionHeader
						title="Notifications"
						description="In-app toasts and activity feed entries."
					/>
				</CardHeader>
				<CardContent className="divide-y divide-border/60">
					<SettingRow
						title="Order fills"
						description="Toast when a paper order fills."
					>
						<Switch checked={notifyFills} onCheckedChange={setNotifyFills} />
					</SettingRow>
					<SettingRow
						title="Price alerts"
						description="Notify when a watchlist level triggers."
					>
						<Switch checked={notifyAlerts} onCheckedChange={setNotifyAlerts} />
					</SettingRow>
					<SettingRow
						title="Strategy errors"
						description="Surface when a bot fails to place an order."
					>
						<Switch checked={notifyBots} onCheckedChange={setNotifyBots} />
					</SettingRow>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<SectionHeader
						title="About"
						description="Build information for this install."
					/>
				</CardHeader>
				<CardContent className="divide-y divide-border/60">
					<SettingRow
						title="Version"
						description="Desktop application version."
					>
						<span className="num text-xs text-muted-foreground">0.1.0</span>
					</SettingRow>
					<SettingRow
						title="Bundle identifier"
						description="Used by Tauri packaging."
					>
						<span className="num text-xs text-muted-foreground">
							com.fatuousnerd.tauri
						</span>
					</SettingRow>
					<SettingRow
						title="Stack"
						description="Shell, UI, and styling layers."
					>
						<span className="text-xs text-muted-foreground">
							Tauri 2 · React 19 · Tailwind 4 · shadcn
						</span>
					</SettingRow>
					<div className="flex flex-wrap items-center gap-2 pt-4">
						<Button asChild variant="outline" size="sm">
							<a href="https://tauri.app" target="_blank" rel="noreferrer">
								Tauri docs
								<ExternalLink className="size-3.5" />
							</a>
						</Button>
						<Button asChild variant="ghost" size="sm">
							<a href="https://ui.shadcn.com" target="_blank" rel="noreferrer">
								shadcn docs
								<ExternalLink className="size-3.5" />
							</a>
						</Button>
					</div>
				</CardContent>
			</Card>

			<p className="flex items-center gap-2 text-[11px] text-muted-foreground">
				<Monitor className="size-3.5" />
				Settings are session-local placeholders — persist them once the config
				layer lands.
			</p>
		</Page>
	);
}
