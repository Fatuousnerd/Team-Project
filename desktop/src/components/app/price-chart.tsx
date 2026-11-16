import { useMemo } from "react";
import {
	Area,
	Bar,
	CartesianGrid,
	ComposedChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";
import { makeCandles } from "@/lib/mock/data";
import { fmtDateTime, fmtPrice } from "@/lib/mock/format";
import type { MarketType, Timeframe } from "@/lib/mock/types";

const AXIS_TICK = { fill: "var(--muted-foreground)", fontSize: 11 };

export function PriceChart({
	symbol,
	market,
	timeframe,
}: {
	symbol: string;
	market: MarketType;
	timeframe: Timeframe;
}) {
	const data = useMemo(
		() => makeCandles(symbol, timeframe),
		[symbol, timeframe],
	);
	const positive =
		data.length > 0 ? data[data.length - 1].close >= data[0].close : true;
	const color = positive ? "var(--positive)" : "var(--negative)";
	const volumeMax = data.reduce(
		(max, candle) => Math.max(max, candle.volume),
		0,
	);

	return (
		<div className="h-[320px] w-full">
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
						minTickGap={48}
						tick={AXIS_TICK}
						tickFormatter={(value) =>
							new Intl.DateTimeFormat("en-US", {
								hour: "2-digit",
								minute: "2-digit",
								hour12: false,
							}).format(Number(value))
						}
					/>
					<YAxis
						yAxisId="price"
						orientation="right"
						axisLine={false}
						tickLine={false}
						width={68}
						tick={AXIS_TICK}
						domain={["dataMin", "dataMax"]}
						tickFormatter={(value) => fmtPrice(Number(value), market)}
					/>
					<YAxis yAxisId="volume" hide domain={[0, volumeMax * 3.6]} />
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
						labelFormatter={(value) => fmtDateTime(Number(value))}
						formatter={(value) => [fmtPrice(Number(value), market), "Price"]}
					/>
					<Bar
						yAxisId="volume"
						dataKey="volume"
						fill="var(--muted-foreground)"
						opacity={0.18}
						isAnimationActive={false}
					/>
					<Area
						yAxisId="price"
						type="monotone"
						dataKey="close"
						stroke={color}
						strokeWidth={1.75}
						fill={color}
						fillOpacity={0.1}
						dot={false}
						isAnimationActive={false}
					/>
				</ComposedChart>
			</ResponsiveContainer>
		</div>
	);
}
