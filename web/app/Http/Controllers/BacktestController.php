<?php

namespace App\Http\Controllers;

use App\Http\Requests\RunBacktestRequest;
use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Services\EngineService;
use Illuminate\Http\Client\RequestException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

class BacktestController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('backtests/index', [
            'results' => auth()->user()->backtestResults()
                ->with('strategy:id,name')
                ->latest('id')
                ->paginate(20),
        ]);
    }

    public function show(BacktestResult $result): Response
    {
        Gate::authorize('view', $result);

        $result->load('strategy:id,name,graph');

        return Inertia::render('backtests/show', [
            'result' => $result,
        ]);
    }

    public function run(RunBacktestRequest $request, Strategy $strategy, EngineService $engine): RedirectResponse
    {
        Gate::authorize('view', $strategy);

        $validated = $request->validated();

        try {
            $engineResult = $engine->runBacktest(
                $strategy->graph,
                $validated['market_filter'] ?? [],
                $validated['date_from'],
                $validated['date_to'],
            );
        } catch (RequestException) {
            return back()->with('error', 'Failed to run backtest. Engine may be unavailable.');
        }

        $trades = collect($engineResult['trades'] ?? []);
        $cumulative = 0.0;
        $transformedTrades = $trades->map(function (array $trade, int $i) use (&$cumulative) {
            $pnl = $trade['pnl_usdc'] ?? 0;
            $cumulative += $pnl;

            return [
                'tick_index' => $i,
                'side' => $trade['side'] ?? 'buy',
                'outcome' => strtoupper($trade['outcome'] ?? 'UP'),
                'entry_price' => $trade['entry_price'] ?? 0,
                'exit_price' => $trade['exit_price'] ?? null,
                'pnl' => round($pnl, 6),
                'cumulative_pnl' => round($cumulative, 6),
                'market_id' => $trade['market_id'] ?? null,
                'entry_at' => $trade['entry_at'] ?? null,
                'exit_at' => $trade['exit_at'] ?? null,
                'exit_reason' => $trade['exit_reason'] ?? null,
            ];
        })->all();

        $result = BacktestResult::create([
            'user_id' => $request->user()->id,
            'strategy_id' => $strategy->id,
            'market_filter' => $validated['market_filter'] ?? null,
            'date_from' => $validated['date_from'],
            'date_to' => $validated['date_to'],
            'total_trades' => $engineResult['total_trades'] ?? null,
            'win_rate' => $engineResult['win_rate'] ?? null,
            'total_pnl_usdc' => $engineResult['total_pnl_usdc'] ?? null,
            'max_drawdown' => $engineResult['max_drawdown'] ?? null,
            'sharpe_ratio' => $engineResult['sharpe_ratio'] ?? null,
            'result_detail' => ['trades' => $transformedTrades],
        ]);

        return to_route('backtests.show', $result)->with('success', 'Backtest completed.');
    }
}
