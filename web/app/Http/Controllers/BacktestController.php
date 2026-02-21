<?php

namespace App\Http\Controllers;

use App\Http\Requests\RunBacktestRequest;
use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Services\EngineService;
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
                ->get(),
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
        $validated = $request->validated();

        $engineResult = $engine->runBacktest(
            $strategy->graph,
            $validated['market_filter'] ?? [],
            $validated['date_from'],
            $validated['date_to'],
        );

        $result = BacktestResult::create([
            'user_id' => $request->user()->id,
            'strategy_id' => $strategy->id,
            'market_filter' => $validated['market_filter'] ?? null,
            'date_from' => $validated['date_from'],
            'date_to' => $validated['date_to'],
            'total_trades' => $engineResult['total_trades'] ?? null,
            'win_rate' => $engineResult['win_rate'] ?? null,
            'total_pnl_usdc' => $engineResult['pnl'] ?? null,
            'max_drawdown' => $engineResult['max_drawdown'] ?? null,
            'sharpe_ratio' => $engineResult['sharpe_ratio'] ?? null,
            'result_detail' => $engineResult,
        ]);

        return to_route('backtests.show', $result)->with('success', 'Backtest completed.');
    }
}
