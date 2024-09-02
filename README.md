# Fiberplane Studio extension for Zed

Real runtime information for your LLM assistant queries.

This extension adds the ability to query local traces from a running [Fiberplane Studio](https://github.com/fiberplane/fpx) instance and add them to your Zed Assistant context.

Traces provide a structured and comprehensive view of the request cycle inside your application. They are a useful tool for debugging and understanding the behavior of your application. As such, they are also a valuable context for making your Zed Assistant queries more effective.

## Installation

Fiberplane Studio extension for Zed is available as a Zed extension. To install it, open `zed: extensions` and search for `Fiberplane Studio`..

## Usage

To use the extension, you need to have a running instance of Fiberplane Studio. You can find more information about how to set up and run Fiberplane Studio in the [Fiberplane Studio documentation](https://fiberplane.com/docs/get-started).

Once you have a running instance of Fiberplane Studio, that has registered some traces, you can use the extension to query those traces in your Zed Assistant queries.

To query a trace simply type `/trace` in you Zed Assistant and then select the trace you want to query. The extension will then fetch and expand the trace in a readable `json` format.
