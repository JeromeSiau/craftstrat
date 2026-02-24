import { Bar, BarChart, CartesianGrid, Cell, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';

interface UpRateBarChartProps {
    data: Array<{ label: string; upRate: number; total: number }>;
    height?: number;
}

export function UpRateBarChart({ data, height = 300 }: UpRateBarChartProps) {
    if (data.length === 0) {
        return <p className="py-8 text-center text-sm text-muted-foreground">No data available.</p>;
    }

    return (
        <ResponsiveContainer width="100%" height={height}>
            <BarChart data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="label" tick={{ fontSize: 12 }} />
                <YAxis domain={[0, 100]} tick={{ fontSize: 12 }} tickFormatter={(v) => `${v}%`} />
                <ReferenceLine y={50} stroke="var(--muted-foreground)" strokeDasharray="4 4" />
                <Tooltip
                    formatter={(value: number, _name: string, props: { payload: { total: number } }) => [
                        `${value.toFixed(1)}% (n=${props.payload.total})`,
                        'UP Rate',
                    ]}
                    contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '0.5rem' }}
                />
                <Bar dataKey="upRate" radius={[4, 4, 0, 0]}>
                    {data.map((entry, idx) => (
                        <Cell
                            key={idx}
                            fill={entry.upRate >= 50 ? '#10b981' : '#ef4444'}
                        />
                    ))}
                </Bar>
            </BarChart>
        </ResponsiveContainer>
    );
}
