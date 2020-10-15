#!/usr/bin/env bash
git log --grep "^Version [0-9].[0-9].[0-9]$" | sed -E "/^(commit|Author)/d" > CHANGELOG
