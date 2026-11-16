import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export function Page({
	children,
	className,
}: {
	children: ReactNode;
	className?: string;
}) {
	return (
		<div
			className={cn(
				"mx-auto flex w-full max-w-[1560px] flex-col gap-5 px-4 py-5 sm:px-6 sm:py-6",
				className,
			)}
		>
			{children}
		</div>
	);
}
