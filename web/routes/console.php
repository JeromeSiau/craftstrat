<?php

use App\Services\EngineService;
use Illuminate\Foundation\Inspiring;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\File;
use Symfony\Component\Console\Command\Command;

Artisan::command('inspire', function () {
    $this->comment(Inspiring::quote());
})->purpose('Display an inspiring quote');

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
