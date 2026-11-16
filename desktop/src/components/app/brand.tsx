import { cn } from "@/lib/utils";

export function BrandMark({ className }: { className?: string }) {
	return (
		<div
			className={cn(
				"flex size-8 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground",
				className,
			)}
		>
			<svg
				viewBox="0 0 20 20"
				className="size-4"
				fill="none"
				aria-hidden="true"
			>
				<path
					d="M4 11.5v4.5M10 4.5v11.5M16 7.5v7"
					stroke="currentColor"
					strokeWidth={2.4}
					strokeLinecap="round"
				/>
			</svg>
		</div>
	);
}

export function Brand({ className }: { className?: string }) {
	return (
		<div className={cn("flex items-center gap-2.5", className)}>
			<BrandMark />
			<div className="flex min-w-0 flex-col leading-none group-data-[collapsible=icon]:hidden">
				<span className="text-sm font-semibold tracking-tight">Meridian</span>
				<span className="mt-0.5 text-[10px] tracking-wide text-muted-foreground uppercase">
					Trading terminal
				</span>
			</div>
		</div>
	);
}
