import { Badge } from "@/components/ui/badge";
import { useMarketStore } from "@/lib/mock/market-store";
import { cn } from "@/lib/utils";

export function ConnectionPill({ className }: { className?: string }) {
	const latency = useMarketStore((state) => state.latency);

	return (
		<Badge
			variant="outline"
			className={cn(
				"h-6 gap-1.5 border-border/80 px-2 text-[11px] font-normal text-muted-foreground",
				className,
			)}
		>
			<span className="relative flex size-1.5">
				<span className="absolute inline-flex size-full animate-ping rounded-full bg-primary opacity-60" />
				<span className="relative inline-flex size-1.5 rounded-full bg-primary" />
			</span>
			<span className="hidden sm:inline">Mock feed</span>
			<span className="num text-muted-foreground/70">{latency} ms</span>
		</Badge>
	);
}
