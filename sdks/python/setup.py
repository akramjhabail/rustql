from setuptools import setup, find_packages

setup(
    name="rustql-python",
    version="0.1.0",
    description="🦀 Official Python SDK for RustQL - World's Fastest API Framework",
    author="akramjhabail",
    license="MIT",
    packages=find_packages(),
    install_requires=[
        "requests>=2.28.0",
        "websocket-client>=1.6.0",
    ],
    python_requires=">=3.8",
    keywords=["rustql", "graphql", "api", "rust", "fast"],
)