import { Link } from '@inertiajs/react';
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import { formatWinRate, formatPnl, pnlColorClass } from '@/lib/formatters';
import type { BacktestResult } from '@/types/models';

interface BacktestResultsTableProps {
    results: BacktestResult[];
    showStrategy?: boolean;
    linkBuilder?: (result: BacktestResult) => string;
}

export default function BacktestResultsTable({
    results,
    showStrategy = false,
    linkBuilder,
}: BacktestResultsTableProps) {
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    {showStrategy && <TableHead>Strategy</TableHead>}
                    <TableHead>Trades</TableHead>
                    <TableHead>Win Rate</TableHead>
                    <TableHead>PnL</TableHead>
                    <TableHead>Date</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {results.map((r) => (
                    <TableRow key={r.id}>
                        {showStrategy && (
                            <TableCell className="font-medium">
                                {linkBuilder ? (
                                    <Link
                                        href={linkBuilder(r)}
                                        className="text-primary hover:underline"
                                    >
                                        {r.strategy.name}
                                    </Link>
                                ) : (
                                    r.strategy.name
                                )}
                            </TableCell>
                        )}
                        <TableCell className="tabular-nums">{r.total_trades ?? '-'}</TableCell>
                        <TableCell className="tabular-nums">{formatWinRate(r.win_rate)}</TableCell>
                        <TableCell className={`tabular-nums font-medium ${pnlColorClass(r.total_pnl_usdc)}`}>
                            {formatPnl(r.total_pnl_usdc)}
                        </TableCell>
                        <TableCell className="text-muted-foreground">
                            {new Date(r.created_at).toLocaleDateString()}
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    );
}
