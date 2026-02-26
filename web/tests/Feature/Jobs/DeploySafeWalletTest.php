<?php

use App\Jobs\DeploySafeWallet;
use App\Models\Wallet;
use Illuminate\Support\Facades\Http;

it('deploys a safe wallet via the engine', function () {
    Http::fake([
        '*/internal/wallet/deploy-safe' => Http::response([
            'safe_address' => '0xSafeAddress1234567890AbCdEf1234567890AbCd',
            'transaction_hash' => '0xTxHash1234',
        ]),
    ]);

    $wallet = Wallet::factory()->pending()->create();

    (new DeploySafeWallet($wallet))->handle(app(\App\Services\EngineService::class));

    $wallet->refresh();

    expect($wallet->status)->toBe('deployed')
        ->and($wallet->safe_address)->toBe('0xSafeAddress1234567890AbCdEf1234567890AbCd')
        ->and($wallet->deployed_at)->not->toBeNull();
});

it('sets status to deploying during execution', function () {
    Http::fake([
        '*/internal/wallet/deploy-safe' => Http::response([
            'safe_address' => '0xSafeAddress1234567890AbCdEf1234567890AbCd',
            'transaction_hash' => '0xTxHash1234',
        ]),
    ]);

    $wallet = Wallet::factory()->pending()->create();

    Http::fake([
        '*/internal/wallet/deploy-safe' => function () use ($wallet) {
            // Verify status is 'deploying' during the HTTP call
            expect($wallet->fresh()->status)->toBe('deploying');

            return Http::response([
                'safe_address' => '0xSafeAddress1234567890AbCdEf1234567890AbCd',
                'transaction_hash' => '0xTxHash1234',
            ]);
        },
    ]);

    (new DeploySafeWallet($wallet))->handle(app(\App\Services\EngineService::class));
});

it('sets status to failed when deployment fails', function () {
    $wallet = Wallet::factory()->pending()->create();

    $job = new DeploySafeWallet($wallet);
    $job->failed(new \RuntimeException('Engine unreachable'));

    expect($wallet->fresh()->status)->toBe('failed');
});
