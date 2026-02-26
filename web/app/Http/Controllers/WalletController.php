<?php

namespace App\Http\Controllers;

use App\Http\Requests\AssignStrategyRequest;
use App\Http\Requests\StoreWalletRequest;
use App\Jobs\DeploySafeWallet;
use App\Models\Strategy;
use App\Models\Wallet;
use App\Services\StrategyActivationService;
use App\Services\WalletService;
use Illuminate\Http\Client\RequestException;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

class WalletController extends Controller
{
    public function index(): Response
    {
        $user = auth()->user();

        return Inertia::render('wallets/index', [
            'wallets' => $user->wallets()
                ->withCount('strategies')
                ->paginate(20),
            'strategies' => $user->strategies()->select('id', 'name')->get(),
        ]);
    }

    public function store(StoreWalletRequest $request, WalletService $walletService): RedirectResponse
    {
        $keypair = $walletService->generateKeypair();

        $wallet = new Wallet([
            'label' => $request->validated('label'),
            'signer_address' => $keypair['signer_address'],
            'status' => 'pending',
        ]);
        $wallet->private_key_enc = $keypair['private_key_enc'];
        $request->user()->wallets()->save($wallet);

        DeploySafeWallet::dispatch($wallet);

        return back()->with('success', 'Wallet creation started. Safe deployment in progress.');
    }

    public function destroy(Wallet $wallet, StrategyActivationService $activation): RedirectResponse
    {
        Gate::authorize('delete', $wallet);

        if ($wallet->isDeploying()) {
            return back()->with('error', 'Cannot delete a wallet while Safe is deploying.');
        }

        try {
            $activation->deactivateAllForWallet($wallet);
        } catch (RequestException) {
            return back()->with('error', 'Failed to deactivate running strategies on engine. Please try again.');
        }

        $wallet->delete();

        return to_route('wallets.index')->with('success', 'Wallet deleted.');
    }

    public function retryDeploy(Wallet $wallet): RedirectResponse
    {
        Gate::authorize('update', $wallet);

        if (! $wallet->isFailed()) {
            return back()->with('error', 'Can only retry failed wallet deployments.');
        }

        $wallet->update(['status' => 'pending']);
        DeploySafeWallet::dispatch($wallet);

        return back()->with('success', 'Retrying Safe deployment.');
    }

    public function assignStrategy(AssignStrategyRequest $request, Wallet $wallet): RedirectResponse
    {
        Gate::authorize('update', $wallet);

        $strategy = Strategy::findOrFail($request->validated('strategy_id'));
        Gate::authorize('view', $strategy);

        $wallet->strategies()->syncWithoutDetaching([
            $strategy->id => [
                'markets' => $request->validated('markets', []),
                'max_position_usdc' => $request->validated('max_position_usdc', 100),
                'is_paper' => $request->validated('is_paper', false),
            ],
        ]);

        return back()->with('success', 'Strategy assigned to wallet.');
    }

    public function removeStrategy(Wallet $wallet, Strategy $strategy): RedirectResponse
    {
        Gate::authorize('update', $wallet);

        $wallet->strategies()->detach($strategy->id);

        return back()->with('success', 'Strategy removed from wallet.');
    }
}
