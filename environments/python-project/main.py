#!/usr/bin/env python3

import requests
import numpy as np
from flask import Flask

def main():
    print("hello from python test project")
    print(f"requests version: {requests.__version__}")
    print(f"numpy version: {np.__version__}")

if __name__ == "__main__":
    main()
