<?php

namespace App\Services;

use App\Models\Trade;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\Relation;

class TradePerformanceService
{
    private const WON_TRADES_SELECT = <<<'SQL'
        SUM(
            CASE
                WHEN status = 'won' THEN 1
                WHEN status = 'closed'
                    AND COALESCE(resolved_price, filled_price, price, 0.5)
                        > COALESCE(filled_price, price, 0.5)
                    THEN 1
                ELSE 0
            END
        ) as won_trades
    SQL;

    private const RESOLVED_TRADES_SELECT = "SUM(CASE WHEN status IN ('won', 'lost', 'closed') THEN 1 ELSE 0 END) as resolved_trades";

    private const TOTAL_PNL_SELECT = <<<'SQL'
        COALESCE(SUM(
            CASE
                WHEN status IN ('won', 'lost', 'closed')
                    THEN (
                        (
                            CASE
                                WHEN status = 'won' THEN COALESCE(resolved_price, 1.0)
                                WHEN status = 'lost' THEN COALESCE(resolved_price, 0.0)
                                ELSE COALESCE(resolved_price, filled_price, price, 0.5)
                            END
                            - COALESCE(filled_price, price, 0.5)
                        )
                        / NULLIF(COALESCE(filled_price, price, 0.5), 0)
                    ) * COALESCE(size_usdc, 0)
                ELSE 0
            END
        ), 0) as total_pnl_usdc
    SQL;

    public function summarizeByStrategyIds(array $strategyIds): array
    {
        if ($strategyIds === []) {
            return [];
        }

        $performanceStats = collect($strategyIds)
            ->mapWithKeys(fn (int $strategyId) => [
                $strategyId => $this->emptySummaryStats(),
            ])
            ->all();

        $aggregates = $this->applyAggregateSelects(
            Trade::query()
                ->whereIn('strategy_id', $strategyIds)
                ->selectRaw('strategy_id, is_paper')
        )
            ->groupBy('strategy_id', 'is_paper')
            ->get();

        foreach ($aggregates as $aggregate) {
            $performanceStats[$aggregate->strategy_id][$aggregate->is_paper ? 'paper' : 'live'] = $this->formatSummaryEntry($aggregate);
        }

        return $performanceStats;
    }

    public function summarizeDetailed(Builder|Relation $query): array
    {
        $aggregate = $this->applyAggregateSelects((clone $query), includeAdvancedMetrics: true)
            ->first();

        return $this->formatDetailedEntry($aggregate);
    }

    public function totalPnlForWalletIds(array $walletIds): string
    {
        if ($walletIds === []) {
            return '0.00';
        }

        $aggregate = $this->applyAggregateSelects(
            Trade::query()->whereIn('wallet_id', $walletIds)
        )->first();

        return number_format((float) ($aggregate?->total_pnl_usdc ?? 0), 2, '.', '');
    }

    public function emptySummaryStats(): array
    {
        return [
            'live' => $this->emptySummaryEntry(),
            'paper' => $this->emptySummaryEntry(),
        ];
    }

    private function applyAggregateSelects(Builder|Relation $query, bool $includeAdvancedMetrics = false): Builder|Relation
    {
        $query
            ->where('side', 'buy')
            ->selectRaw('COUNT(*) as total_trades')
            ->selectRaw(self::WON_TRADES_SELECT)
            ->selectRaw(self::RESOLVED_TRADES_SELECT)
            ->selectRaw(self::TOTAL_PNL_SELECT);

        if ($includeAdvancedMetrics) {
            $query
                ->selectRaw('AVG(fill_slippage_bps) as avg_fill_slippage_bps')
                ->selectRaw('AVG(markout_bps_60s) as avg_markout_bps_60s');
        }

        return $query;
    }

    private function formatSummaryEntry(?object $aggregate): array
    {
        $resolvedTrades = (int) ($aggregate?->resolved_trades ?? 0);

        return [
            'total_trades' => (int) ($aggregate?->total_trades ?? 0),
            'win_rate' => $resolvedTrades > 0
                ? number_format((int) $aggregate->won_trades / $resolvedTrades, 4, '.', '')
                : null,
            'total_pnl_usdc' => number_format((float) ($aggregate?->total_pnl_usdc ?? 0), 2, '.', ''),
        ];
    }

    private function formatDetailedEntry(?object $aggregate): array
    {
        $entry = $this->formatSummaryEntry($aggregate);

        $entry['avg_fill_slippage_bps'] = $aggregate?->avg_fill_slippage_bps !== null
            ? number_format((float) $aggregate->avg_fill_slippage_bps, 2, '.', '')
            : null;
        $entry['avg_markout_bps_60s'] = $aggregate?->avg_markout_bps_60s !== null
            ? number_format((float) $aggregate->avg_markout_bps_60s, 2, '.', '')
            : null;

        return $entry;
    }

    private function emptySummaryEntry(): array
    {
        return [
            'total_trades' => 0,
            'win_rate' => null,
            'total_pnl_usdc' => '0.00',
        ];
    }
}
