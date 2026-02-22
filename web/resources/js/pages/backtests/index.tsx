import { Head } from '@inertiajs/react';
import { LineChart } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent } from '@/components/ui/card';
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
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">Backtest Results</h1>
                    <p className="mt-1 text-muted-foreground">
                        Review historical performance of your strategies.
                    </p>
                </div>

                {results.data.length === 0 ? (
                    <Card>
                        <CardContent className="flex flex-col items-center justify-center py-16 text-center">
                            <div className="rounded-xl bg-muted p-4">
                                <LineChart className="size-8 text-muted-foreground" />
                            </div>
                            <p className="mt-4 font-medium">No backtest results yet</p>
                            <p className="mt-1 text-sm text-muted-foreground">
                                Run a backtest from a strategy page to see results here.
                            </p>
                        </CardContent>
                    </Card>
                ) : (
                    <Card className="border-l-4 border-l-amber-500/50">
                        <CardContent className="pt-6">
                            <BacktestResultsTable
                                results={results.data}
                                showStrategy
                                linkBuilder={(r) => show.url(r.id)}
                            />
                        </CardContent>
                    </Card>
                )}
            </div>
        </AppLayout>
    );
}
