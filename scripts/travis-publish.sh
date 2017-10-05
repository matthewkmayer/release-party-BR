#!/bin/bash

# To speed up CI builds, only build with release if we're on a tag.
if [ -n "$TRAVIS_TAG" ] ; then 
    cargo build --release;
    rm /target/release/release-party-br.d ; # extra bits from compiler we don't need
    if [[ $TRAVIS_OS_NAME == "osx" ]] ; then
        echo "moving it for osx" ; 
        ls ./target/release/ ; 
        file ./target/release/release-party-br ;
        mv ./target/release/release-party-br ./target/release/release-party-br-darwin-amd64 ;
    fi
    if [[ $TRAVIS_OS_NAME == "linux" ]] ; then
        echo "moving it for linux" ; 
        ls ./target/release/ ; 
        file ./target/release/release-party-br ; 
        mv ./target/release/release-party-br ./target/release/release-party-br-linux-amd64 ; 
    fi
fi
