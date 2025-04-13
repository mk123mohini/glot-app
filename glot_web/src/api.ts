export { Response, run, sendLoginLink };

interface Response {
  body: string;
  status: number;
}

async function run(data: any): Promise<unknown> {
  const response = await fetch("/internal-api/run", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });

  return response.json();
}

async function sendLoginLink(data: any): Promise<Response> {
  const response = await fetch("/internal-api/magiclink/send", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });

  const body = await response.text();

  return {
    body: body,
    status: response.status,
  };
}
