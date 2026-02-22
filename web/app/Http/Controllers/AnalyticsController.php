<?php

namespace App\Http\Controllers;

use App\Services\EngineService;
use Illuminate\Http\Client\RequestException;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;

class AnalyticsController extends Controller
{
    public function index(Request $request, EngineService $engine): Response
    {
        $slotDuration = (int) $request->query('slot_duration', 900);
        $symbols = $request->query('symbols')
            ? array_filter(explode(',', $request->query('symbols')))
            : [];
        $hours = (float) $request->query('hours', 168.0);

        try {
            $stats = $engine->slotStats($slotDuration, $symbols, $hours);
        } catch (RequestException) {
            $stats = null;
        }

        return Inertia::render('analytics/index', [
            'stats' => $stats,
            'filters' => [
                'slot_duration' => $slotDuration,
                'symbols' => $symbols,
                'hours' => $hours,
            ],
        ]);
    }
}
