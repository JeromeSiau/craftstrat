<?php

use App\Services\EngineService;
use App\Services\MlTrainerService;
use Illuminate\Foundation\Inspiring;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\File;
use Symfony\Component\Console\Command\Command;

Artisan::command('inspire', function () {
    $this->comment(Inspiring::quote());
})->purpose('Display an inspiring quote');

Artisan::command('ml:trainer-status', function (MlTrainerService $trainer) {
    $payload = $trainer->health();

    $this->line(json_encode($payload, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_THROW_ON_ERROR));

    return Command::SUCCESS;
})->purpose('Show ML trainer health, defaults, and latest candidate metadata');

Artisan::command('ml:refresh-candidate
    {--slot-duration= : Override slot duration}
    {--symbols= : Override comma-separated symbols, e.g. BTC,ETH}
    {--hours= : Override lookback window in hours}
    {--sample-every= : Override keep-one-row-every-N-snapshots}
    {--limit= : Override engine page size}
    {--max-rows= : Optional cap on exported rows}
    {--verbose-eval= : Override XGBoost verbose eval frequency}
    {--rl-gamma= : Override RL-like discount factor}', function (MlTrainerService $trainer) {
    $payload = array_filter([
        'slot_duration' => $this->option('slot-duration'),
        'symbols' => $this->option('symbols'),
        'hours' => $this->option('hours'),
        'sample_every' => $this->option('sample-every'),
        'limit' => $this->option('limit'),
        'max_rows' => $this->option('max-rows'),
        'verbose_eval' => $this->option('verbose-eval'),
        'rl_gamma' => $this->option('rl-gamma'),
    ], fn ($value) => $value !== null);

    $report = $trainer->refreshCandidate($payload);

    $this->info(sprintf(
        'Candidate %s trained with %d rows.',
        $report['candidate_name'] ?? 'unknown',
        (int) ($report['export']['rows'] ?? 0),
    ));
    $this->line(sprintf('Candidate dir: %s', $report['candidate_dir'] ?? 'n/a'));
    $this->line(sprintf(
        'Policy min_edge: %.4f',
        (float) ($report['candidate']['policy']['min_edge'] ?? 0.0),
    ));
    $this->line(sprintf(
        'Entry min_value: %.4f',
        (float) ($report['candidate']['rl_like']['entry_policy']['recommended']['min_value'] ?? 0.0),
    ));

    return Command::SUCCESS;
})->purpose('Export a fresh ML dataset and train a candidate bundle through the ML trainer');

Artisan::command('ml:promote-candidate {candidate_name? : Candidate directory name to promote}', function (MlTrainerService $trainer) {
    $report = $trainer->promoteCandidate($this->argument('candidate_name'));

    $this->info(sprintf(
        'Promoted %s into %s',
        $report['promoted_from'] ?? 'unknown',
        $report['live_dir'] ?? 'n/a',
    ));

    if (! empty($report['backup_dir'])) {
        $this->line(sprintf('Backup dir: %s', $report['backup_dir']));
    }

    return Command::SUCCESS;
})->purpose('Promote a candidate ML bundle into the live model path');

Artisan::command('ml:export-slot-dataset {slot_duration}
    {--symbols= : Comma-separated symbols, e.g. BTC,ETH}
    {--hours=720 : Lookback window in hours}
    {--sample-every=1 : Keep one row every N snapshots per slot}
    {--limit=10000 : Page size per engine request}
    {--path=storage/app/ml/slot-dataset.ndjson : Output file path} ', function (EngineService $engine) {
    $slotDuration = (int) $this->argument('slot_duration');
    $symbols = collect(explode(',', (string) $this->option('symbols')))
        ->map(fn (string $symbol) => trim($symbol))
        ->filter()
        ->values()
        ->all();
    $hours = (float) $this->option('hours');
    $sampleEvery = (int) $this->option('sample-every');
    $limit = (int) $this->option('limit');
    $offset = 0;
    $total = 0;

    $path = (string) $this->option('path');
    $absolutePath = str_starts_with($path, DIRECTORY_SEPARATOR)
        ? $path
        : base_path($path);

    File::ensureDirectoryExists(dirname($absolutePath));

    $handle = fopen($absolutePath, 'wb');
    if ($handle === false) {
        $this->error("Unable to open output file: {$absolutePath}");

        return Command::FAILURE;
    }

    try {
        do {
            $payload = $engine->slotMlDataset(
                slotDuration: $slotDuration,
                symbols: $symbols,
                hours: $hours,
                sampleEvery: $sampleEvery,
                limit: $limit,
                offset: $offset,
            );

            $rows = $payload['rows'] ?? [];

            foreach ($rows as $row) {
                fwrite($handle, json_encode($row, JSON_UNESCAPED_SLASHES | JSON_THROW_ON_ERROR).PHP_EOL);
            }

            $count = count($rows);
            $total += $count;
            $offset += $count;

            $this->info("Exported {$total} rows...");
        } while ($count === $limit);
    } catch (Throwable $e) {
        fclose($handle);
        File::delete($absolutePath);
        $this->error($e->getMessage());

        return Command::FAILURE;
    }

    fclose($handle);

    $this->info("Dataset export completed: {$total} rows -> {$absolutePath}");

    return Command::SUCCESS;
})->purpose('Export slot ML dataset as NDJSON through the engine internal API');
