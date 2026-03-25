<?php

namespace App\Http\Controllers;

use App\Exceptions\StrategyGenerationException;
use App\Http\Requests\GenerateStrategyRequest;
use App\Http\Requests\StoreStrategyRequest;
use App\Http\Requests\UpdateStrategyRequest;
use App\Models\Strategy;
use App\Services\EngineService;
use App\Services\StrategyActivationService;
use App\Services\StrategyGeneratorService;
use App\Services\TradePerformanceService;
use Illuminate\Http\Client\RequestException;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

class StrategyController extends Controller
{
    public function index(TradePerformanceService $performance): Response
    {
        $strategies = auth()->user()->strategies()
            ->withCount('wallets')
            ->with('walletStrategies:id,strategy_id,markets')
            ->latest()
            ->paginate(20);

        $performanceStats = $performance->summarizeByStrategyIds(
            $strategies->getCollection()->pluck('id')->all()
        );

        $strategies->getCollection()->transform(function (Strategy $strategy) use ($performance, $performanceStats) {
            $strategy->setAttribute(
                'performance_stats',
                $performanceStats[$strategy->id] ?? $performance->emptySummaryStats()
            );

            return $strategy;
        });

        return Inertia::render('strategies/index', [
            'strategies' => $strategies,
        ]);
    }

    public function create(): Response
    {
        return Inertia::render('strategies/create');
    }

    public function generate(GenerateStrategyRequest $request, StrategyGeneratorService $generator): JsonResponse
    {
        try {
            $result = $generator->generate($request->validated('description'));

            return response()->json(['graph' => $result['graph']]);
        } catch (StrategyGenerationException $e) {
            return response()->json(['error' => $e->getMessage()], 422);
        }
    }

    public function store(StoreStrategyRequest $request): RedirectResponse
    {
        $request->user()->strategies()->create($request->validated());

        return to_route('strategies.index')->with('success', 'Strategy created.');
    }

    public function edit(Strategy $strategy): Response
    {
        Gate::authorize('update', $strategy);

        return Inertia::render('strategies/edit', [
            'strategy' => $strategy,
        ]);
    }

    public function show(Strategy $strategy): Response
    {
        Gate::authorize('view', $strategy);

        $strategy->load(['walletStrategies.wallet', 'backtestResults' => fn ($q) => $q->latest('id')->limit(5)]);

        return Inertia::render('strategies/show', [
            'strategy' => $strategy,
            'liveStats' => Inertia::defer(function () use ($strategy) {
                return [
                    'live' => app(TradePerformanceService::class)->summarizeDetailed($strategy->liveTrades()),
                    'paper' => app(TradePerformanceService::class)->summarizeDetailed($strategy->paperTrades()),
                ];
            }, 'liveData'),
            'recentTrades' => Inertia::defer(fn () => $strategy->trades()
                ->orderByRaw('COALESCE(executed_at, created_at) DESC')
                ->limit(20)
                ->get([
                    'id',
                    'symbol',
                    'side',
                    'outcome',
                    'price',
                    'reference_price',
                    'filled_price',
                    'resolved_price',
                    'fill_slippage_bps',
                    'markout_bps_60s',
                    'size_usdc',
                    'status',
                    'is_paper',
                    'executed_at',
                    'created_at',
                    'markout_at_60s',
                ]), 'liveData'),
        ]);
    }

    public function update(UpdateStrategyRequest $request, Strategy $strategy): RedirectResponse
    {
        $strategy->update($request->validated());

        return back()->with('success', 'Strategy updated.');
    }

    public function destroy(Strategy $strategy, StrategyActivationService $activation): RedirectResponse
    {
        Gate::authorize('delete', $strategy);

        try {
            $activation->deactivateAllForStrategy($strategy);
        } catch (RequestException) {
            return back()->with('error', 'Failed to deactivate strategy on engine. Please try again.');
        }

        $strategy->delete();

        return to_route('strategies.index')->with('success', 'Strategy deleted.');
    }

    public function activate(Strategy $strategy, StrategyActivationService $activation): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        try {
            $activation->activate($strategy);
        } catch (\RuntimeException $e) {
            return back()->with('error', $e->getMessage());
        } catch (RequestException) {
            return back()->with('error', 'Failed to activate strategy. Engine may be unavailable.');
        }

        return back()->with('success', 'Strategy activated.');
    }

    public function deactivate(Strategy $strategy, StrategyActivationService $activation): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        try {
            $activation->deactivate($strategy);
        } catch (RequestException) {
            return back()->with('error', 'Failed to deactivate strategy. Engine may be unavailable.');
        }

        return back()->with('success', 'Strategy deactivated.');
    }

    public function kill(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        try {
            $strategy->load('walletStrategies');
            foreach ($strategy->walletStrategies as $ws) {
                $engine->killStrategy($ws->wallet_id, $strategy->id);
            }
        } catch (RequestException) {
            return back()->with('error', 'Failed to kill strategy. Engine may be unavailable.');
        }

        return back()->with('success', 'Kill switch activated — all evaluation stopped.');
    }

    public function unkill(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        try {
            $strategy->load('walletStrategies');
            foreach ($strategy->walletStrategies as $ws) {
                $engine->unkillStrategy($ws->wallet_id, $strategy->id);
            }
        } catch (RequestException) {
            return back()->with('error', 'Failed to resume strategy. Engine may be unavailable.');
        }

        return back()->with('success', 'Kill switch deactivated — evaluation resumed.');
    }
}
