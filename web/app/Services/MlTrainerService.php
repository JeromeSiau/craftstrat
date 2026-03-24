<?php

namespace App\Services;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Facades\Http;

class MlTrainerService
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly int $timeout,
    ) {}

    /**
     * @return array<string, mixed>
     */
    public function health(): array
    {
        return $this->client()
            ->get('/health')
            ->throw()
            ->json();
    }

    /**
     * @param  array<string, mixed>  $payload
     * @return array<string, mixed>
     */
    public function refreshCandidate(array $payload = []): array
    {
        return $this->client()
            ->timeout($this->timeout * 30)
            ->post('/refresh', $payload)
            ->throw()
            ->json();
    }

    /**
     * @return array<string, mixed>
     */
    public function promoteCandidate(?string $candidateName = null): array
    {
        return $this->client()
            ->timeout($this->timeout * 10)
            ->post('/promote', array_filter([
                'candidate_name' => $candidateName,
            ], fn ($value) => $value !== null))
            ->throw()
            ->json();
    }

    private function client(): PendingRequest
    {
        return Http::baseUrl($this->baseUrl)
            ->timeout($this->timeout)
            ->acceptJson();
    }
}
