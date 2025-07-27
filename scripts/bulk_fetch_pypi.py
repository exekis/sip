#!/usr/bin/env python3
"""
bulk pypi metadata fetcher for sip registry

usage: python3 bulk_fetch_pypi.py packages.txt > python_packages.json

where packages.txt contains one package name per line.
"""

import requests
import json
import sys
from datetime import date
from typing import List, Dict, Any

def fetch_pypi_metadata(pkg_name: str, trust_score: float = 5.0) -> Dict[str, Any]:
    """fetch metadata for a single package from pypi"""
    try:
        # get pypi json
        resp = requests.get(f"https://pypi.org/pypi/{pkg_name}/json", timeout=10)
        resp.raise_for_status()
        meta = resp.json()

        # pick the latest version
        version = meta["info"]["version"]
        files = meta["releases"].get(version, [])
        if not files:
            raise ValueError(f"no release files found for {pkg_name}=={version}")

        # find the sdist (or fall back to the first file)
        sdist = next((f for f in files if f["packagetype"] == "sdist"), files[0])
        sha256_from_pypi = sdist["digests"]["sha256"]

        # determine source url
        source_url = None
        if meta["info"].get("project_urls"):
            project_urls = meta["info"]["project_urls"]
            source_url = (
                project_urls.get("Homepage") or
                project_urls.get("Source") or
                project_urls.get("Repository") or
                project_urls.get("Source Code")
            )
        
        if not source_url:
            source_url = meta["info"].get("home_page")
        
        if not source_url or source_url == "UNKNOWN":
            source_url = f"https://pypi.org/project/{pkg_name}/"

        # build record matching sip format
        record = {
            "name": pkg_name,
            "version": version,
            "hash": f"sha256:{sha256_from_pypi}",
            "trust_score": trust_score,
            "endorsed_by": ["pypi-bulk-fetch"],
            "last_reviewed": date.today().isoformat(),
            "source": source_url
        }
        
        return record
        
    except Exception as e:
        print(f"error fetching {pkg_name}: {e}", file=sys.stderr)
        raise

def main():
    if len(sys.argv) != 2:
        print("usage: python3 bulk_fetch_pypi.py packages.txt", file=sys.stderr)
        sys.exit(1)
    
    packages_file = sys.argv[1]
    
    try:
        with open(packages_file, 'r') as f:
            package_names = [
                line.strip() 
                for line in f 
                if line.strip() and not line.strip().startswith('#')
            ]
    except FileNotFoundError:
        print(f"error: file '{packages_file}' not found", file=sys.stderr)
        sys.exit(1)
    
    if not package_names:
        print("error: no packages found in file", file=sys.stderr)
        sys.exit(1)
    
    print(f"fetching metadata for {len(package_names)} packages...", file=sys.stderr)
    
    results = []
    success_count = 0
    error_count = 0
    
    for i, pkg_name in enumerate(package_names):
        print(f"({i+1}/{len(package_names)}) fetching {pkg_name}...", file=sys.stderr)
        try:
            record = fetch_pypi_metadata(pkg_name)
            results.append(record)
            success_count += 1
        except Exception:
            error_count += 1
    
    print(f"\ncompleted: {success_count} success, {error_count} errors", file=sys.stderr)
    
    # output json to stdout
    json.dump(results, sys.stdout, indent=2)

if __name__ == "__main__":
    main()
