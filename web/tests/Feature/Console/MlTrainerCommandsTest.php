<?php

use Illuminate\Support\Facades\Http;

beforeEach(function () {
    $this->withoutVite();
});

it('shows trainer status', function () {
    Http::fake([
        'http://ml-trainer:8011/health' => Http::response([
            'ok' => true,
            'model_name' => 'btc-15m-xgb-policy',
        ]),
    ]);

    $this->artisan('ml:trainer-status')
        ->expectsOutputToContain('"ok": true')
        ->assertSuccessful();
});

it('refreshes a candidate bundle through the trainer service', function () {
    Http::fake([
        'http://ml-trainer:8011/refresh' => Http::response([
            'candidate_name' => 'btc-15m-xgb-policy-candidate-20260324-120000',
            'candidate_dir' => '/models/candidates/btc-15m-xgb-policy-candidate-20260324-120000',
            'export' => ['rows' => 12345],
            'candidate' => [
                'policy' => ['min_edge' => 0.09],
                'rl_like' => [
                    'entry_policy' => [
                        'recommended' => ['min_value' => 0.01],
                    ],
                ],
            ],
        ]),
    ]);

    $this->artisan('ml:refresh-candidate', [
        '--symbols' => 'BTC',
        '--sample-every' => '6',
    ])
        ->expectsOutputToContain('Candidate btc-15m-xgb-policy-candidate-20260324-120000 trained with 12345 rows.')
        ->assertSuccessful();

    Http::assertSent(function ($request) {
        return $request->url() === 'http://ml-trainer:8011/refresh'
            && $request['symbols'] === 'BTC'
            && $request['sample_every'] === '6';
    });
});

it('promotes a candidate bundle through the trainer service', function () {
    Http::fake([
        'http://ml-trainer:8011/promote' => Http::response([
            'promoted_from' => 'btc-15m-xgb-policy-candidate-20260324-120000',
            'live_dir' => '/models/btc-15m-xgb-policy',
            'backup_dir' => '/models/backups/btc-15m-xgb-policy-20260324-121500',
        ]),
    ]);

    $this->artisan('ml:promote-candidate', [
        'candidate_name' => 'btc-15m-xgb-policy-candidate-20260324-120000',
    ])
        ->expectsOutputToContain('Promoted btc-15m-xgb-policy-candidate-20260324-120000 into /models/btc-15m-xgb-policy')
        ->assertSuccessful();

    Http::assertSent(function ($request) {
        return $request->url() === 'http://ml-trainer:8011/promote'
            && $request['candidate_name'] === 'btc-15m-xgb-policy-candidate-20260324-120000';
    });
});
