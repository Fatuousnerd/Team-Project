import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { allocationByMarket } from "@/lib/mock/derive";
import { fmtMoney } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";

const COLORS = ["var(--chart-1)", "var(--chart-4)", "var(--chart-5)"];

export function AllocationChart() {
	const positions = useMarketStore((state) => state.positions);
	const quotes = useMarketStore((state) => state.quotes);
	const data = allocationByMarket(positions, quotes);
	const total = data.reduce((sum, entry) => sum + entry.value, 0);

	if (data.length === 0) {
		return (
			<div className="flex h-[220px] items-center justify-center text-xs text-muted-foreground">
				No open exposure
			</div>
		);
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="relative h-[172px]">
				<ResponsiveContainer width="100%" height="100%">
					<PieChart>
						<Pie
							data={data}
							dataKey="value"
							nameKey="name"
							innerRadius={54}
							outerRadius={78}
							paddingAngle={2}
							stroke="var(--card)"
							strokeWidth={2}
							isAnimationActive={false}
						>
							{data.map((entry, index) => (
								<Cell
									key={entry.name}
									fill={COLORS[index % COLORS.length]}
									className="outline-none"
								/>
							))}
						</Pie>
						<Tooltip
							contentStyle={{
								background: "var(--popover)",
								border: "1px solid var(--border)",
								borderRadius: "calc(var(--radius) * 1.2)",
								fontSize: 12,
								color: "var(--popover-foreground)",
							}}
							formatter={(value) => [
								fmtMoney(Number(value), { whole: true }),
								"Exposure",
							]}
						/>
					</PieChart>
				</ResponsiveContainer>
				<div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-0.5">
					<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
						Exposure
					</span>
					<span className="num text-base font-semibold">
						{fmtMoney(total, { whole: true })}
					</span>
				</div>
			</div>

			<div className="flex flex-col gap-2">
				{data.map((entry, index) => (
					<div key={entry.name} className="flex items-center gap-2 text-xs">
						<span
							className="size-2 shrink-0 rounded-[2px]"
							style={{ backgroundColor: COLORS[index % COLORS.length] }}
						/>
						<span className="text-muted-foreground">{entry.name}</span>
						<span className="num ms-auto font-medium">
							{fmtMoney(entry.value, { whole: true })}
						</span>
						<span className="num w-11 text-end text-muted-foreground">
							{total === 0 ? "0.0" : ((entry.value / total) * 100).toFixed(1)}%
						</span>
					</div>
				))}
			</div>
		</div>
	);
}
