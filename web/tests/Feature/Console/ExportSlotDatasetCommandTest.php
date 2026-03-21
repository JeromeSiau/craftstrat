<?php

use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Http;

beforeEach(function () {
    $this->withoutVite();
});

it('exports slot ml dataset as ndjson', function () {
    $path = storage_path('app/testing/slot-dataset.ndjson');
    File::delete($path);

    Http::fake([
        '*/internal/stats/slots/ml-dataset*' => Http::sequence()
            ->push([
                'row_count' => 1,
                'rows' => [[
                    'captured_at' => '2026-03-20T00:00:00Z',
                    'symbol' => 'btc-updown-15m-1700000000',
                    'slot_ts' => 1700000000,
                    'slot_duration' => 900,
                    'target_up' => 1,
                    'f_mid_up' => 0.61,
                ]],
            ])
            ->push([
                'row_count' => 0,
                'rows' => [],
            ]),
    ]);

    $this->artisan('ml:export-slot-dataset', [
        'slot_duration' => 900,
        '--limit' => 1,
        '--path' => $path,
    ])->assertSuccessful();

    expect(File::exists($path))->toBeTrue();

    $lines = file($path, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    expect($lines)->toHaveCount(1);

    $payload = json_decode($lines[0], true, 512, JSON_THROW_ON_ERROR);
    expect($payload)
        ->toHaveKey('symbol', 'btc-updown-15m-1700000000')
        ->toHaveKey('target_up', 1);

    Http::assertSentCount(2);

    File::delete($path);
});
