import { ACTIVITY_ICONS } from "@/components/app/command-palette";
import { fmtAgo } from "@/lib/mock/format";
import type { Activity } from "@/lib/mock/types";

export function ActivityList({ items }: { items: Activity[] }) {
	return (
		<div className="flex flex-col">
			{items.map((item, index) => {
				const Icon = ACTIVITY_ICONS[item.kind];
				return (
					<div
						key={item.id}
						className={
							index === items.length - 1
								? "flex items-start gap-3 py-3"
								: "flex items-start gap-3 border-b border-border/60 py-3"
						}
					>
						<div className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-secondary text-secondary-foreground">
							<Icon className="size-3.5" />
						</div>
						<div className="flex min-w-0 flex-1 flex-col gap-0.5">
							<span className="truncate text-xs font-medium">{item.title}</span>
							<span className="truncate text-[11px] text-muted-foreground">
								{item.detail}
							</span>
						</div>
						<span className="shrink-0 text-[10px] text-muted-foreground/70">
							{fmtAgo(item.at)}
						</span>
					</div>
				);
			})}
		</div>
	);
}
