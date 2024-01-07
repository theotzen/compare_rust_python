# Repository Overview

This repository serves as a comprehensive collection of two distinct backend applications, one implemented in Rust and the other in Python. The purpose of this repository is to provide a practical comparison of the performance and efficiency of these two popular programming languages in a controlled backend task. The accompanying article, which details the findings of our comparison, aims to offer insights into the strengths and trade-offs of each language in backend development.

## Contents

The repository is structured into two main directories:

### `rust_app`

The `rust_app` directory contains the Rust implementation of the backend application. It utilizes the Rocket framework to create a web server that handles specific tasks related to GitHub content fetching and YAML file comparison. The application is designed to demonstrate Rust's capabilities in terms of speed, memory usage, and CPU efficiency.

Key features of the `rust_app` include:

- Use of the [Rocket framework]([url](https://rocket.rs/)) for handling HTTP requests and responses.
- Implementation of the [Octocrab crate]([url](https://github.com/XAMPPRocky/octocrab)) to interact with the GitHub API.
- Efficient algorithms for comparing YAML configurations using Rust's powerful type system and performance-oriented features.

### `python_app`

The `python_app` directory houses the Python version of the backend application. It leverages the FastAPI framework to provide similar functionality to the Rust application, focusing on ease of development and rapid prototyping.

Highlights of the `python_app` include:

- [FastAPI framework]([url](https://fastapi.tiangolo.com/)) for building APIs with Python 3.7+.
- Utilization of the [PyGithub library]([url](https://github.com/PyGithub/PyGithub)) for GitHub API communication.
- Simple and readable code for fetching and comparing YAML files, showcasing Python's developer-friendly syntax.

## Purpose of the Article

The article associated with this repository delves into the results of our performance comparison between the Rust and Python applications. It presents a series of tests and benchmarks that were conducted to evaluate the execution time, CPU usage, and memory consumption of both implementations when performing identical tasks.

The article does not advocate for one language over the other but instead provides factual data and observations from our experiments. It aims to inform developers about the potential impact of their choice of language and framework on the performance of their backend services.

By sharing our methodology, test results, and analysis, we hope to contribute to the ongoing discussion about the suitability of Rust and Python for various backend development scenarios. The repository and article together serve as a resource for developers who are considering these languages for their projects and wish to make an informed decision based on real-world performance metrics.

## Conclusion

Whether you are a seasoned developer or new to backend development, this repository and the accompanying article offer valuable perspectives on the practical implications of using Rust or Python for backend tasks. We encourage you to explore the code, read the article, and consider the findings in the context of your own development needs and preferences.
