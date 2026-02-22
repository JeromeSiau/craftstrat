import { Head } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import BacktestResultsTable from '@/components/backtest-results-table';
import type { BreadcrumbItem } from '@/types';
import type { BacktestResult, Paginated } from '@/types/models';
import { index, show } from '@/actions/App/Http/Controllers/BacktestController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Backtests', href: index.url() },
];

export default function BacktestsIndex({ results }: { results: Paginated<BacktestResult> }) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Backtests" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Backtest Results</h1>
                {results.data.length === 0 ? (
                    <p className="text-muted-foreground">
                        No backtest results yet. Run one from a strategy page.
                    </p>
                ) : (
                    <BacktestResultsTable
                        results={results.data}
                        showStrategy
                        linkBuilder={(r) => show.url(r.id)}
                    />
                )}
            </div>
        </AppLayout>
    );
}
