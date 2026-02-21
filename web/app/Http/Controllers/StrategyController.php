<?php

namespace App\Http\Controllers;

use App\Http\Requests\StoreStrategyRequest;
use App\Http\Requests\UpdateStrategyRequest;
use App\Models\Strategy;
use App\Services\EngineService;
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
                ->get(),
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

        $strategy->load(['walletStrategies.wallet', 'backtestResults' => fn ($q) => $q->latest()->limit(5)]);

        return Inertia::render('strategies/show', [
            'strategy' => $strategy,
        ]);
    }

    public function update(UpdateStrategyRequest $request, Strategy $strategy): RedirectResponse
    {
        $strategy->update($request->validated());

        return back()->with('success', 'Strategy updated.');
    }

    public function destroy(Strategy $strategy): RedirectResponse
    {
        Gate::authorize('delete', $strategy);

        $strategy->delete();

        return to_route('strategies.index')->with('success', 'Strategy deleted.');
    }

    public function activate(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        $runningAssignments = $strategy->walletStrategies()->where('is_running', false)->with('wallet')->get();

        foreach ($runningAssignments as $assignment) {
            $engine->activateStrategy(
                $assignment->wallet_id,
                $strategy->id,
                $strategy->graph,
                $assignment->markets ?? [],
                (float) $assignment->max_position_usdc,
            );

            $assignment->update(['is_running' => true, 'started_at' => now()]);
        }

        $strategy->update(['is_active' => true]);

        return back()->with('success', 'Strategy activated.');
    }

    public function deactivate(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        Gate::authorize('update', $strategy);

        $runningAssignments = $strategy->walletStrategies()->where('is_running', true)->get();

        foreach ($runningAssignments as $assignment) {
            $engine->deactivateStrategy($assignment->wallet_id, $strategy->id);
            $assignment->update(['is_running' => false, 'started_at' => null]);
        }

        $strategy->update(['is_active' => false]);

        return back()->with('success', 'Strategy deactivated.');
    }
}
