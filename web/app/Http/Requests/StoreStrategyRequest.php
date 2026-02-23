<?php

namespace App\Http\Requests;

use App\Http\Requests\Concerns\ValidatesApiFetchNodes;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreStrategyRequest extends FormRequest
{
    use ValidatesApiFetchNodes;

    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'description' => ['nullable', 'string'],
            'graph' => ['required', 'array'],
            'graph.mode' => ['required', 'in:form,node'],
            'mode' => ['required', 'in:form,node', 'same:graph.mode'],
        ];
    }

    /**
     * @return array<int, callable>
     */
    public function after(): array
    {
        return [
            fn ($validator) => $this->validateApiFetchNodes($validator),
        ];
    }
}
