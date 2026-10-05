#!/usr/bin/env python3
"""The existing package-scoped claim address shared by research transforms."""

import hashlib
import re


def normalise(text):
    return re.sub(r"\s+", " ", text.lower().strip()).rstrip(" .;:,!")


def claim_id(scope, text):
    return "claim-" + hashlib.sha256(f"{scope}:{normalise(text)}".encode("utf-8")).hexdigest()[:16]
