import { Bar, BarChart, CartesianGrid, Cell, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';

interface WinRateBarChartProps {
    data: Array<{ label: string; winRate: number; total: number }>;
    height?: number;
}

export function WinRateBarChart({ data, height = 300 }: WinRateBarChartProps) {
    if (data.length === 0) {
        return <p className="py-8 text-center text-sm text-muted-foreground">No data available.</p>;
    }

    return (
        <ResponsiveContainer width="100%" height={height}>
            <BarChart data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="label" tick={{ fontSize: 12 }} />
                <YAxis domain={[0, 100]} tick={{ fontSize: 12 }} tickFormatter={(v) => `${v}%`} />
                <ReferenceLine y={50} stroke="hsl(var(--muted-foreground))" strokeDasharray="4 4" />
                <Tooltip
                    formatter={(value: number, _name: string, props: { payload: { total: number } }) => [
                        `${value.toFixed(1)}% (n=${props.payload.total})`,
                        'Win Rate',
                    ]}
                    contentStyle={{ background: 'hsl(var(--background))', border: '1px solid hsl(var(--border))' }}
                />
                <Bar dataKey="winRate" radius={[4, 4, 0, 0]}>
                    {data.map((entry, idx) => (
                        <Cell
                            key={idx}
                            fill={entry.winRate >= 50 ? 'hsl(var(--chart-2))' : 'hsl(var(--chart-5))'}
                        />
                    ))}
                </Bar>
            </BarChart>
        </ResponsiveContainer>
    );
}
