<?php

namespace App\Jobs;

use App\Models\Wallet;
use App\Services\EngineService;
use Illuminate\Contracts\Queue\ShouldBeUnique;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Foundation\Queue\Queueable;
use Throwable;

class DeploySafeWallet implements ShouldBeUnique, ShouldQueue
{
    use Queueable;

    public int $tries = 3;

    /** @var list<int> */
    public array $backoff = [10, 30, 60];

    public int $uniqueFor = 300;

    public function __construct(public readonly Wallet $wallet) {}

    public function uniqueId(): int
    {
        return $this->wallet->id;
    }

    public function handle(EngineService $engine): void
    {
        $this->wallet->update(['status' => 'deploying']);

        $result = $engine->deploySafe(
            $this->wallet->id,
            $this->wallet->signer_address,
            $this->wallet->private_key_enc,
        );

        $this->wallet->update([
            'safe_address' => $result['safe_address'],
            'status' => 'deployed',
            'deployed_at' => now(),
        ]);
    }

    public function failed(Throwable $exception): void
    {
        $this->wallet->update(['status' => 'failed']);
    }
}
