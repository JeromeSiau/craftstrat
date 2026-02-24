import { Head, router } from '@inertiajs/react';
import { ArrowDownRight, ArrowUpRight, Clock, Database, Layers, TrendingUp } from 'lucide-react';
import { index as analyticsIndex } from '@/actions/App/Http/Controllers/AnalyticsController';
import { CalibrationChart } from '@/components/charts/calibration-chart';
import { StoplossSweepChart } from '@/components/charts/stoploss-sweep-chart';
import { UpRateBarChart } from '@/components/charts/up-rate-bar-chart';
import { UpRateHeatmap } from '@/components/charts/up-rate-heatmap';
import MetricCard from '@/components/metric-card';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';
import type { AnalyticsFilters, SlotAnalyticsData } from '@/types/models';

interface Props {
    stats: SlotAnalyticsData | null;
    filters: AnalyticsFilters;
}

const DURATION_OPTIONS = [
    { value: '300', label: '5 min' },
    { value: '900', label: '15 min' },
    { value: '3600', label: '1 hour' },
    { value: '14400', label: '4 hours' },
    { value: '86400', label: '1 day' },
];

const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

function formatCompact(n: number): string {
    if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 10_000) return `${(n / 1_000).toFixed(1)}k`;
    return n.toLocaleString();
}

function formatDataAge(lastSnapshotAt: string | null): string {
    if (!lastSnapshotAt) return '-';
    const now = Date.now();
    const then = new Date(lastSnapshotAt + 'Z').getTime();
    const diffMs = now - then;
    if (diffMs < 0) return 'just now';
    const totalMinutes = Math.floor(diffMs / 60000);
    if (totalMinutes < 1) return '<1m ago';
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    if (hours === 0) return `${minutes}m ago`;
    return `${hours}h ${minutes}m ago`;
}

function applyFilters(filters: AnalyticsFilters, overrides: Partial<AnalyticsFilters>): void {
    const merged = { ...filters, ...overrides };
    const params: Record<string, string | number> = {
        slot_duration: merged.slot_duration,
        hours: merged.hours,
    };
    if (merged.symbols.length > 0) {
        params.symbols = merged.symbols.join(',');
    }
    router.get(analyticsIndex.url(), params, { preserveState: true, preserveScroll: true });
}

const breadcrumbs: BreadcrumbItem[] = [{ title: 'Analytics', href: analyticsIndex.url() }];

export default function AnalyticsIndex({ stats, filters }: Props) {
    if (!stats) {
        return (
            <AppLayout breadcrumbs={breadcrumbs}>
                <Head title="Analytics" />
                <div className="p-4 md:p-8">
                    <Card>
                        <CardContent className="flex items-center justify-center py-16">
                            <p className="text-muted-foreground">
                                Unable to load analytics data. The engine may be unavailable.
                            </p>
                        </CardContent>
                    </Card>
                </div>
            </AppLayout>
        );
    }

    const { summary, heatmap, calibration, by_symbol, stoploss_sweep, by_hour, by_day } = stats;

    // Collect all unique symbols from by_symbol data
    const allSymbols = by_symbol.map((s) => s.symbol);

    const toggleSymbol = (symbol: string) => {
        let next: string[];
        if (filters.symbols.length === 0) {
            // All are selected; clicking one means "only this one"
            next = [symbol];
        } else if (filters.symbols.includes(symbol)) {
            next = filters.symbols.filter((s) => s !== symbol);
        } else {
            next = [...filters.symbols, symbol];
        }
        applyFilters(filters, { symbols: next });
    };

    const symbolBarData = by_symbol.map((s) => ({
        label: s.symbol,
        upRate: s.win_rate * 100,
        total: s.total,
    }));

    const hourBarData = by_hour.map((h) => ({
        label: `${h.period}h`,
        upRate: h.win_rate * 100,
        total: h.total,
    }));

    const dayBarData = by_day.map((d) => ({
        label: DAY_LABELS[d.period] ?? `${d.period}`,
        upRate: d.win_rate * 100,
        total: d.total,
    }));

    const overallUpRate =
        summary.resolved_slots > 0
            ? by_symbol.reduce((acc, s) => acc + s.wins, 0) / by_symbol.reduce((acc, s) => acc + s.total, 0)
            : null;

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Analytics" />
            <div className="space-y-6 p-4 md:p-8">
                {/* Header + Filters */}
                <div className="flex flex-wrap items-center gap-4">
                    <h1 className="text-2xl font-bold tracking-tight">Slot Analytics</h1>
                    <div className="ml-auto flex flex-wrap items-center gap-3">
                        <Select
                            value={String(filters.slot_duration)}
                            onValueChange={(v) => applyFilters(filters, { slot_duration: Number(v) })}
                        >
                            <SelectTrigger className="w-[130px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {DURATION_OPTIONS.map((opt) => (
                                    <SelectItem key={opt.value} value={opt.value}>
                                        {opt.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        {allSymbols.length > 0 && (
                            <div className="flex flex-wrap gap-1">
                                {allSymbols.map((sym) => {
                                    const isActive =
                                        filters.symbols.length === 0 || filters.symbols.includes(sym);
                                    return (
                                        <button
                                            key={sym}
                                            onClick={() => toggleSymbol(sym)}
                                            className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-colors ${
                                                isActive
                                                    ? 'border-primary bg-primary text-primary-foreground'
                                                    : 'border-border bg-background text-muted-foreground hover:bg-accent'
                                            }`}
                                        >
                                            {sym}
                                        </button>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </div>

                {/* KPI Row */}
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    <MetricCard label="Total Slots" value={summary.total_slots.toLocaleString()} icon={Layers} accent="blue" />
                    <MetricCard label="Resolved" value={summary.resolved_slots.toLocaleString()} icon={TrendingUp} accent="emerald" />
                    <MetricCard label="In Progress" value={summary.unresolved_slots.toLocaleString()} icon={Clock} accent="amber" />
                    <MetricCard
                        label="Snapshots"
                        value={formatCompact(summary.total_snapshots)}
                        icon={Database}
                        accent="violet"
                    />
                    <MetricCard
                        label="UP Rate"
                        value={overallUpRate !== null ? `${(overallUpRate * 100).toFixed(1)}%` : '-'}
                        icon={overallUpRate !== null && overallUpRate >= 0.5 ? ArrowUpRight : ArrowDownRight}
                        accent={overallUpRate !== null && overallUpRate >= 0.5 ? 'emerald' : 'red'}
                    />
                </div>

                {/* Heatmap */}
                <Card>
                    <CardHeader>
                        <CardTitle>UP Rate Heatmap (Time vs Move)</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <UpRateHeatmap data={heatmap} />
                    </CardContent>
                </Card>

                {/* Calibration + By Symbol */}
                <div className="grid gap-6 lg:grid-cols-2">
                    <Card>
                        <CardHeader>
                            <CardTitle>Calibration</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <CalibrationChart data={calibration} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader>
                            <CardTitle>UP Rate by Symbol</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <UpRateBarChart data={symbolBarData} />
                        </CardContent>
                    </Card>
                </div>

                {/* Stoploss Sweep */}
                <Card>
                    <CardHeader>
                        <CardTitle>Stoploss Sweep</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <StoplossSweepChart data={stoploss_sweep} />
                    </CardContent>
                </Card>

                {/* By Hour + By Day */}
                <div className="grid gap-6 lg:grid-cols-2">
                    <Card>
                        <CardHeader>
                            <CardTitle>UP Rate by Hour (UTC)</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <UpRateBarChart data={hourBarData} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader>
                            <CardTitle>UP Rate by Day</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <UpRateBarChart data={dayBarData} />
                        </CardContent>
                    </Card>
                </div>

                {/* Footer Summary */}
                <p className="text-center text-sm text-muted-foreground">
                    {summary.resolved_slots.toLocaleString()} resolved slots
                    {' '}&middot; last {filters.hours}h window
                </p>
            </div>
        </AppLayout>
    );
}
