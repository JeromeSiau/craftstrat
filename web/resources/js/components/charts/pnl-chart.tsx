import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import type { BacktestTrade } from '@/types/models';

interface PnlChartProps {
    trades: BacktestTrade[];
}

export function PnlChart({ trades }: PnlChartProps) {
    const data = trades.map((t, i) => ({
        trade: i + 1,
        pnl: t.cumulative_pnl,
    }));

    return (
        <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis dataKey="trade" tick={{ fontSize: 12 }} />
                <YAxis tick={{ fontSize: 12 }} tickFormatter={(v) => `$${v}`} />
                <Tooltip
                    formatter={(value: number) => [`$${value.toFixed(2)}`, 'Cumulative PnL']}
                    contentStyle={{ background: 'var(--background)', border: '1px solid var(--border)', borderRadius: '0.5rem' }}
                />
                <Area
                    type="monotone"
                    dataKey="pnl"
                    stroke="var(--chart-1)"
                    fill="var(--chart-1)"
                    fillOpacity={0.2}
                />
            </AreaChart>
        </ResponsiveContainer>
    );
}
