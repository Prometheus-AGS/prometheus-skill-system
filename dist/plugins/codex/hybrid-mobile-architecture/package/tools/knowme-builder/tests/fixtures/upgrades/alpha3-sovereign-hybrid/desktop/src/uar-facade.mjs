export class ServiceUarRuntimeFacade {
  constructor({ endpoint, fetchImpl = fetch }) {
    this.endpoint = endpoint;
    this.fetchImpl = fetchImpl;
  }

  async run(request, signal) {
    const response = await this.fetchImpl(`${this.endpoint}/api/v1/runs`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(request),
      signal,
    });
    if (!response.ok) throw new Error(`UAR run failed: ${response.status}`);
    return response.json();
  }
}
