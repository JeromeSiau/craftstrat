<?php

namespace App\Http\Requests\Concerns;

use Illuminate\Validation\Validator;

trait ValidatesApiFetchNodes
{
    protected function validateApiFetchNodes(Validator $validator): void
    {
        $graph = $this->input('graph');
        if (! is_array($graph) || ($graph['mode'] ?? '') !== 'node') {
            return;
        }

        $nodes = collect($graph['nodes'] ?? [])
            ->filter(fn ($node) => ($node['type'] ?? '') === 'api_fetch');

        if ($nodes->count() > 5) {
            $validator->errors()->add('graph.nodes', 'A strategy may contain at most 5 API Fetch nodes.');

            return;
        }

        foreach ($nodes->values() as $i => $node) {
            $data = $node['data'] ?? [];
            $url = trim($data['url'] ?? '');
            $interval = $data['interval_secs'] ?? 60;

            if ($url === '') {
                $validator->errors()->add("graph.nodes.{$i}.data.url", 'API Fetch nodes require a URL.');

                continue;
            }

            if (! str_starts_with($url, 'https://')) {
                $validator->errors()->add("graph.nodes.{$i}.data.url", 'API Fetch URLs must use HTTPS.');
            }

            if ($this->isPrivateUrl($url)) {
                $validator->errors()->add("graph.nodes.{$i}.data.url", 'API Fetch URLs must not point to private or internal addresses.');
            }

            if ((int) $interval < 30) {
                $validator->errors()->add("graph.nodes.{$i}.data.interval_secs", 'API Fetch interval must be at least 30 seconds.');
            }
        }
    }

    private function isPrivateUrl(string $url): bool
    {
        $host = parse_url($url, PHP_URL_HOST);
        if ($host === null || $host === false) {
            return true;
        }

        $lower = strtolower($host);

        // Block localhost aliases
        if (in_array($lower, ['localhost', '0.0.0.0', '[::1]'], true)) {
            return true;
        }

        // If host is an IP literal, check for private/reserved ranges
        if (filter_var($host, FILTER_VALIDATE_IP)) {
            return ! filter_var($host, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE);
        }

        // Block internal-looking hostnames
        if (str_ends_with($lower, '.local') || str_ends_with($lower, '.internal')) {
            return true;
        }

        return false;
    }
}
