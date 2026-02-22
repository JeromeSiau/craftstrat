<?php

namespace App\Http\Controllers;

use App\Http\Requests\StoreStrategyRequest;
use App\Http\Requests\UpdateStrategyRequest;
use App\Models\Strategy;
use App\Services\StrategyActivationService;
use Illuminate\Http\Client\RequestException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

class StrategyController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('strategies/index', [
            'strategies' => auth()->user()->strategies()
                ->withCount('wallets')
                ->latest()
                ->paginate(20),
        ]);
    }

    public function create(): Response
    {
        return Inertia::render('strategies/create');
    }

    public function store(StoreStrategyRequest $request): RedirectResponse
    {
        $request->user()->strategies()->create($request->validated());

        return to_route('strategies.index')->with('success', 'Strategy created.');
    }

    public function show(Strategy $strategy): Response
    {
        Gate::authorize('view', $strategy);

        $strategy->load(['walletStrategies.wallet', 'backtestResults' => fn ($q) => $q->latest('id')->limit(5)]);

        return Inertia::render('strategies/show', [
            'strategy' => $strategy,
            'liveStats' => Inertia::defer(function () use ($strategy) {
                $filled = $strategy->trades()->where('status', 'filled');
                $totalTrades = $filled->count();
                $totalPnl = (float) $filled->sum('size_usdc');
                $winCount = $strategy->trades()
                    ->where('status', 'filled')
                    ->where('price', '>', 0.5)
                    ->count();

                return [
                    'total_trades' => $totalTrades,
                    'win_rate' => $totalTrades > 0
                        ? number_format($winCount / $totalTrades, 4)
                        : null,
                    'total_pnl_usdc' => number_format($totalPnl, 2, '.', ''),
                ];
            }, 'liveData'),
            'recentTrades' => Inertia::defer(fn () => $strategy->trades()
                ->latest('executed_at')
                ->limit(20)
                ->get(['id', 'market_id', 'side', 'outcome', 'price', 'size_usdc', 'status', 'executed_at']), 'liveData'),
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
}
