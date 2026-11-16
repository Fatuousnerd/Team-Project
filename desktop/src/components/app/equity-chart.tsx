import { useMemo } from "react";
import {
	Area,
	CartesianGrid,
	ComposedChart,
	Line,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { type EquityRange, makeEquityCurve } from "@/lib/mock/data";
import { fmtCompact, fmtMoney } from "@/lib/mock/format";

const AXIS_TICK = { fill: "var(--muted-foreground)", fontSize: 11 };

export function EquityChart({ range }: { range: EquityRange }) {
	const data = useMemo(() => makeEquityCurve(range), [range]);
	const { min, max } = useMemo(() => {
		const values = data.flatMap((point) => [point.value, point.benchmark]);
		const low = Math.min(...values);
		const high = Math.max(...values);
		const pad = Math.max(250, (high - low) * 0.12);
		return { min: low - pad, max: high + pad };
	}, [data]);

	return (
		<div className="h-[260px] w-full">
			<ResponsiveContainer width="100%" height="100%">
				<ComposedChart
					data={data}
					margin={{ top: 8, right: 4, left: 4, bottom: 0 }}
				>
					<CartesianGrid
						vertical={false}
						stroke="var(--border)"
						strokeDasharray="3 3"
					/>
					<XAxis
						dataKey="time"
						axisLine={false}
						tickLine={false}
						minTickGap={56}
						tick={AXIS_TICK}
						tickFormatter={(value) =>
							range === "1D"
								? new Intl.DateTimeFormat("en-US", {
										hour: "2-digit",
										minute: "2-digit",
										hour12: false,
									}).format(Number(value))
								: new Intl.DateTimeFormat("en-US", {
										month: "short",
										day: "numeric",
									}).format(Number(value))
						}
					/>
					<YAxis
						orientation="right"
						axisLine={false}
						tickLine={false}
						width={56}
						tick={AXIS_TICK}
						domain={[min, max]}
						tickFormatter={(value) => `$${fmtCompact(Number(value))}`}
					/>
					<Tooltip
						cursor={{ stroke: "var(--border)" }}
						contentStyle={{
							background: "var(--popover)",
							border: "1px solid var(--border)",
							borderRadius: "calc(var(--radius) * 1.2)",
							fontSize: 12,
							color: "var(--popover-foreground)",
							boxShadow: "0 8px 24px oklch(0 0 0 / 0.14)",
						}}
						labelStyle={{ color: "var(--muted-foreground)", marginBottom: 4 }}
						labelFormatter={(value) =>
							new Intl.DateTimeFormat("en-US", {
								month: "short",
								day: "numeric",
								hour: "2-digit",
								minute: "2-digit",
								hour12: false,
							}).format(Number(value))
						}
						formatter={(value, name) => [
							fmtMoney(Number(value), { whole: true }),
							name === "value" ? "Portfolio" : "Benchmark",
						]}
					/>
					<Area
						type="monotone"
						dataKey="value"
						stroke="var(--chart-1)"
						strokeWidth={1.9}
						fill="var(--chart-1)"
						fillOpacity={0.1}
						dot={false}
						isAnimationActive={false}
					/>
					<Line
						type="monotone"
						dataKey="benchmark"
						stroke="var(--muted-foreground)"
						strokeWidth={1.4}
						strokeDasharray="4 4"
						dot={false}
						isAnimationActive={false}
					/>
				</ComposedChart>
			</ResponsiveContainer>
		</div>
	);
}
