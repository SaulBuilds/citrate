"""
Setup script for Citrate Python SDK
"""

from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="citrate-sdk",
    version="0.1.0",
    author="Citrate Team",
    author_email="developers@citrate.ai",
    description="Python SDK for Citrate AI blockchain platform",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/citrate-ai/citrate-v3",
    project_urls={
        "Documentation": "https://docs.citrate.ai",
        "Source": "https://github.com/citrate-ai/citrate-v3",
        "Tracker": "https://github.com/citrate-ai/citrate-v3/issues",
    },
    packages=find_packages(),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: System :: Distributed Computing",
    ],
    python_requires=">=3.8",
    install_requires=requirements,
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-asyncio",
            "black",
            "flake8",
            "mypy",
            "isort",
        ],
        "docs": [
            "sphinx",
            "sphinx-rtd-theme",
            "myst-parser",
        ],
    },
    entry_points={
        "console_scripts": [
            "citrate=citrate_sdk.cli:main",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)