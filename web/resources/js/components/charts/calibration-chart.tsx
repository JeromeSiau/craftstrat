import {
    CartesianGrid,
    Cell,
    ReferenceLine,
    ResponsiveContainer,
    Scatter,
    ScatterChart,
    Tooltip,
    XAxis,
    YAxis,
    ZAxis,
} from 'recharts';
import type { CalibrationPoint } from '@/types/models';

interface CalibrationChartProps {
    data: CalibrationPoint[];
}

export function CalibrationChart({ data }: CalibrationChartProps) {
    if (data.length === 0) {
        return (
            <p className="py-8 text-center text-sm text-muted-foreground">
                No calibration data available.
            </p>
        );
    }

    const chartData = data.map((d) => ({
        x: d.avg_bid * 100,
        y: d.win_rate * 100,
        sampleCount: d.sample_count,
    }));

    return (
        <ResponsiveContainer width="100%" height={300}>
            <ScatterChart margin={{ top: 10, right: 10, bottom: 10, left: 0 }}>
                <CartesianGrid
                    strokeDasharray="3 3"
                    className="stroke-border"
                />
                <XAxis
                    type="number"
                    dataKey="x"
                    name="Market P(Up)"
                    domain={[10, 95]}
                    tick={{ fontSize: 12 }}
                    tickFormatter={(v: number) => `${Math.round(v)}%`}
                    label={{
                        value: 'Market P(Up) %',
                        position: 'insideBottom',
                        offset: -5,
                        fontSize: 12,
                    }}
                />
                <YAxis
                    type="number"
                    dataKey="y"
                    name="Actual UP Rate"
                    domain={[10, 95]}
                    tick={{ fontSize: 12 }}
                    tickFormatter={(v: number) => `${Math.round(v)}%`}
                    label={{
                        value: 'Actual UP %',
                        angle: -90,
                        position: 'insideLeft',
                        fontSize: 12,
                    }}
                />
                <ZAxis type="number" dataKey="sampleCount" range={[40, 400]} />
                <ReferenceLine
                    segment={[
                        { x: 10, y: 10 },
                        { x: 95, y: 95 },
                    ]}
                    stroke="var(--muted-foreground)"
                    strokeDasharray="4 4"
                    label={{
                        value: 'Perfect calibration',
                        position: 'insideTopLeft',
                        fontSize: 11,
                    }}
                />
                <Tooltip
                    cursor={{ strokeDasharray: '3 3' }}
                    formatter={(
                        value: number | undefined,
                        name: string | undefined,
                    ) => {
                        const displayValue = value ?? 0;
                        if (name === 'Market P(Up)')
                            return [`${displayValue.toFixed(1)}%`, name];
                        if (name === 'Actual UP Rate')
                            return [`${displayValue.toFixed(1)}%`, name];
                        return [displayValue, name ?? 'Value'];
                    }}
                    contentStyle={{
                        background: 'var(--background)',
                        border: '1px solid var(--border)',
                        borderRadius: '0.5rem',
                    }}
                />
                <Scatter data={chartData}>
                    {chartData.map((entry, idx) => (
                        <Cell
                            key={idx}
                            fill={entry.y >= entry.x ? '#10b981' : '#ef4444'}
                        />
                    ))}
                </Scatter>
            </ScatterChart>
        </ResponsiveContainer>
    );
}
