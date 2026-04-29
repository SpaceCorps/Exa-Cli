using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;

namespace Exa.Console.Infrastructure;

public sealed class ExaClient : IDisposable
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull
    };

    private readonly HttpClient _http;

    public ExaClient(string apiKey)
    {
        _http = new HttpClient { BaseAddress = new Uri("https://api.exa.ai/") };
        _http.DefaultRequestHeaders.Add("x-api-key", apiKey);
        _http.DefaultRequestHeaders.Accept.Add(
            new MediaTypeWithQualityHeaderValue("application/json"));
    }

    public async Task<JsonDocument> PostAsync(string path, object body)
    {
        var response = await _http.PostAsJsonAsync(path, body, JsonOptions);
        return await HandleResponseAsync(response);
    }

    private static async Task<JsonDocument> HandleResponseAsync(HttpResponseMessage response)
    {
        var body = await response.Content.ReadAsStringAsync();

        if (!response.IsSuccessStatusCode)
        {
            var message = TryExtractError(body)
                ?? $"HTTP {(int)response.StatusCode}: {body}";
            throw new HttpRequestException(message);
        }

        if (string.IsNullOrWhiteSpace(body))
            return JsonDocument.Parse("{}");

        return JsonDocument.Parse(body);
    }

    private static string? TryExtractError(string body)
    {
        try
        {
            using var doc = JsonDocument.Parse(body);
            if (doc.RootElement.TryGetProperty("error", out var error))
                return error.GetString();
        }
        catch { }
        return null;
    }

    public void Dispose() => _http.Dispose();
}
