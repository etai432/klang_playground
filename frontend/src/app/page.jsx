
export default async function Home() {
  console.log("starting");
  let response = await fetch("http://127.0.0.1:8000/api/run", {
    method: "POST",
    headers: {
      'Content-Type': 'application/json'
    }
  });

  const myJson = await response.json();

  console.log(myJson);
  return (

    <h1>hello world</h1>

  )
}
