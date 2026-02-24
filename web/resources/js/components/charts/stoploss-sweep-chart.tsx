import {
    Bar,
    CartesianGrid,
    ComposedChart,
    Line,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from 'recharts';
import type { StoplossThreshold } from '@/types/models';

interface StoplossSweepChartProps {
    data: StoplossThreshold[];
}

export function StoplossSweepChart({ data }: StoplossSweepChartProps) {
    if (data.length === 0) {
        return <p className="py-8 text-center text-sm text-muted-foreground">No stoploss sweep data available.</p>;
    }

    const chartData = data.map((d) => ({
        threshold: d.threshold,
        trueSaves: d.true_saves,
        falseExits: d.false_exits,
        precision: d.precision * 100,
    }));

    return (
        <ResponsiveContainer width="100%" height={300}>
            <ComposedChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis
                    dataKey="threshold"
                    tick={{ fontSize: 12 }}
                    tickFormatter={(v) => v.toFixed(2)}
                />
                <YAxis yAxisId="left" tick={{ fontSize: 12 }} label={{ value: 'Count', angle: -90, position: 'insideLeft', fontSize: 12 }} />
                <YAxis
                    yAxisId="right"
                    orientation="right"
                    domain={[0, 100]}
                    tick={{ fontSize: 12 }}
                    tickFormatter={(v) => `${v}%`}
                    label={{ value: 'Precision %', angle: 90, position: 'insideRight', fontSize: 12 }}
                />
                <Tooltip
                    contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '0.5rem' }}
                    formatter={(value: number, name: string) => {
                        if (name === 'precision') return [`${value.toFixed(1)}%`, 'Precision'];
                        if (name === 'trueSaves') return [value, 'True Saves'];
                        if (name === 'falseExits') return [value, 'False Exits'];
                        return [value, name];
                    }}
                    labelFormatter={(label) => `Threshold: ${Number(label).toFixed(2)}`}
                />
                <Bar
                    yAxisId="left"
                    dataKey="trueSaves"
                    stackId="a"
                    fill="#10b981"
                    radius={[0, 0, 0, 0]}
                />
                <Bar
                    yAxisId="left"
                    dataKey="falseExits"
                    stackId="a"
                    fill="#ef4444"
                    radius={[4, 4, 0, 0]}
                />
                <Line
                    yAxisId="right"
                    type="monotone"
                    dataKey="precision"
                    stroke="#8b5cf6"
                    strokeWidth={2}
                    dot={false}
                />
            </ComposedChart>
        </ResponsiveContainer>
    );
}
