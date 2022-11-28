window.BENCHMARK_DATA = 
{
  "lastUpdate": 1669607507771,
  "repoUrl": "https://github.com/hydro-project/hydroflow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c7a7304291dea4fd8fd95509c17b6aa1d4ea029c",
          "message": "fixup! Update CI for new index.md",
          "timestamp": "2021-10-30T00:20:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c7a7304291dea4fd8fd95509c17b6aa1d4ea029c"
        },
        "date": 1635553707322,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375679,
            "range": "± 2779",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 203020318,
            "range": "± 942138",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 188451017,
            "range": "± 1640198",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8449248,
            "range": "± 44861",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41778614,
            "range": "± 190313",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 43210330,
            "range": "± 395969",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12952216,
            "range": "± 8883",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2318677,
            "range": "± 3391",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2349046,
            "range": "± 2204",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8399cf953a2563110977cd61c750fe8308f54a92",
          "message": "Implement reachability/hydroflow (scheduled + compiled) benchmark",
          "timestamp": "2021-10-30T00:39:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8399cf953a2563110977cd61c750fe8308f54a92"
        },
        "date": 1635555313000,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375647,
            "range": "± 2908",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 189152213,
            "range": "± 790008",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 182167295,
            "range": "± 1577713",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9399075,
            "range": "± 21406",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46056758,
            "range": "± 104428",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47882792,
            "range": "± 337911",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14676732,
            "range": "± 12000",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2582580,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2585103,
            "range": "± 1997",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8399cf953a2563110977cd61c750fe8308f54a92",
          "message": "Implement reachability/hydroflow (scheduled + compiled) benchmark",
          "timestamp": "2021-10-30T00:39:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8399cf953a2563110977cd61c750fe8308f54a92"
        },
        "date": 1635555922725,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 442207,
            "range": "± 6134",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 212331529,
            "range": "± 1121643",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 213734991,
            "range": "± 2432352",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11164260,
            "range": "± 176971",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 56509458,
            "range": "± 3421375",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56616409,
            "range": "± 1097022",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17743046,
            "range": "± 1049780",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3063127,
            "range": "± 48540",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3046803,
            "range": "± 37017",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8399cf953a2563110977cd61c750fe8308f54a92",
          "message": "Implement reachability/hydroflow (scheduled + compiled) benchmark",
          "timestamp": "2021-10-30T00:39:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8399cf953a2563110977cd61c750fe8308f54a92"
        },
        "date": 1635565697726,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 391735,
            "range": "± 18957",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200863567,
            "range": "± 4144721",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 192958632,
            "range": "± 6767139",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9414247,
            "range": "± 404288",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 48966351,
            "range": "± 2541011",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50325679,
            "range": "± 3289662",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15669888,
            "range": "± 761113",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2737481,
            "range": "± 117040",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2715808,
            "range": "± 116018",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8399cf953a2563110977cd61c750fe8308f54a92",
          "message": "Implement reachability/hydroflow (scheduled + compiled) benchmark",
          "timestamp": "2021-10-30T00:39:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8399cf953a2563110977cd61c750fe8308f54a92"
        },
        "date": 1635652084444,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375761,
            "range": "± 2816",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 202092415,
            "range": "± 563865",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 193003411,
            "range": "± 2192581",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9431773,
            "range": "± 15649",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47063629,
            "range": "± 188646",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48263527,
            "range": "± 355608",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681844,
            "range": "± 7898",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2564455,
            "range": "± 3510",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2576607,
            "range": "± 2895",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8399cf953a2563110977cd61c750fe8308f54a92",
          "message": "Implement reachability/hydroflow (scheduled + compiled) benchmark",
          "timestamp": "2021-10-30T00:39:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8399cf953a2563110977cd61c750fe8308f54a92"
        },
        "date": 1635738504544,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375679,
            "range": "± 2939",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196343875,
            "range": "± 1227717",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 195172065,
            "range": "± 2304817",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9345891,
            "range": "± 26936",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47631627,
            "range": "± 152823",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48418903,
            "range": "± 338575",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685187,
            "range": "± 29852",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2672987,
            "range": "± 2518",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2593610,
            "range": "± 2511",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "id": "c22ed90898193d05436d6743ed282338755d03f7",
          "message": "Add TeeingHandoff",
          "timestamp": "2021-10-29T21:58:32Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c22ed90898193d05436d6743ed282338755d03f7"
        },
        "date": 1635791856227,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 389310,
            "range": "± 15953",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 193643352,
            "range": "± 2884312",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 194953275,
            "range": "± 4838714",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 81623672,
            "range": "± 1597776",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9537620,
            "range": "± 405853",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 52154485,
            "range": "± 1292359",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50485401,
            "range": "± 2072238",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15244301,
            "range": "± 718444",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2723170,
            "range": "± 111775",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2821681,
            "range": "± 108703",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "id": "df21b5d61e524f71f58d502a60a203e6817617f4",
          "message": "split out handoffs",
          "timestamp": "2021-11-01T19:16:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/df21b5d61e524f71f58d502a60a203e6817617f4"
        },
        "date": 1635824893552,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375642,
            "range": "± 2635",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 185909925,
            "range": "± 277878",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 181779076,
            "range": "± 2324279",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 57969395,
            "range": "± 217161",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11908413,
            "range": "± 75621",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44655375,
            "range": "± 93963",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48238780,
            "range": "± 332151",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14674819,
            "range": "± 10872",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2554236,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2562939,
            "range": "± 2224",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "2e3eb331c796a9347b6b69f87c68f2a8e51c1306",
          "message": "Add [ci-bench] tag to trigger CI benchmark on push",
          "timestamp": "2021-11-02T10:49:23-07:00",
          "tree_id": "5fa89012d3b52c03c6e179d5776daf9e46bb5b9d",
          "url": "https://github.com/hydro-project/hydroflow/commit/2e3eb331c796a9347b6b69f87c68f2a8e51c1306"
        },
        "date": 1635875900887,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375637,
            "range": "± 2734",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198990009,
            "range": "± 969183",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 197408439,
            "range": "± 2149795",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 58293374,
            "range": "± 637446",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8643096,
            "range": "± 15054",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 45835993,
            "range": "± 159493",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48377676,
            "range": "± 1238118",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679331,
            "range": "± 13158",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2602169,
            "range": "± 4748",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2634837,
            "range": "± 14829",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "43ee3ba5eb43eec8c1f745bc0fcdbaab24c8b544",
          "message": "Add scheduling from external events [ci-bench]",
          "timestamp": "2021-11-02T17:39:18-07:00",
          "tree_id": "663ebf0e8ef3009d5a4d87eab404923d3547571c",
          "url": "https://github.com/hydro-project/hydroflow/commit/43ee3ba5eb43eec8c1f745bc0fcdbaab24c8b544"
        },
        "date": 1635900491775,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 289933,
            "range": "± 8499",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 194009194,
            "range": "± 7346502",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 196374047,
            "range": "± 8411504",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 55322556,
            "range": "± 1804335",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10571947,
            "range": "± 315289",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43984444,
            "range": "± 1413400",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48090379,
            "range": "± 1465975",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 13447248,
            "range": "± 427528",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2486712,
            "range": "± 65478",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2504562,
            "range": "± 79149",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "43ee3ba5eb43eec8c1f745bc0fcdbaab24c8b544",
          "message": "Add scheduling from external events [ci-bench]",
          "timestamp": "2021-11-03T00:39:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43ee3ba5eb43eec8c1f745bc0fcdbaab24c8b544"
        },
        "date": 1635911322935,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374938,
            "range": "± 2708",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195809189,
            "range": "± 847005",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 193276982,
            "range": "± 7379308",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 59748749,
            "range": "± 609697",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9410961,
            "range": "± 25570",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 45618128,
            "range": "± 1728252",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47962525,
            "range": "± 392320",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684229,
            "range": "± 8354",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2587617,
            "range": "± 2845",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2584035,
            "range": "± 1817",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "justin.jaffray@gmail.com",
            "name": "Justin Jaffray",
            "username": "justinj"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6d424e7d6474b049f604f2de6855897591536505",
          "message": "Extend Covid tracing demo (#4)\n\nThis commit extends the Covid tracing demo to run in real-time. Includes a\r\nbunch of randomly generated data to give it some texture.",
          "timestamp": "2021-11-03T13:41:21-07:00",
          "tree_id": "8c61f5637b9eab45b53f67f9dfa592b52cdd5eeb",
          "url": "https://github.com/hydro-project/hydroflow/commit/6d424e7d6474b049f604f2de6855897591536505"
        },
        "date": 1635972729834,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375270,
            "range": "± 2707",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204742143,
            "range": "± 2942681",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 206982129,
            "range": "± 2839406",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 60655494,
            "range": "± 989345",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9439682,
            "range": "± 25274",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46115710,
            "range": "± 1847212",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48979169,
            "range": "± 567073",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14692228,
            "range": "± 7714",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2612959,
            "range": "± 63455",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2586284,
            "range": "± 2399",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "b1f4d69adf9176173b33cd0ede41c6f6c1ded831",
          "message": "Split scheduled module into smaller pieces",
          "timestamp": "2021-11-03T14:08:22-07:00",
          "tree_id": "59d7a921deb6015f15b037aa697f9e6e2b082488",
          "url": "https://github.com/hydro-project/hydroflow/commit/b1f4d69adf9176173b33cd0ede41c6f6c1ded831"
        },
        "date": 1635974247842,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374809,
            "range": "± 2573",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196596146,
            "range": "± 433211",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 193832066,
            "range": "± 10762725",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 57965932,
            "range": "± 328030",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13104551,
            "range": "± 53254",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46054287,
            "range": "± 1837529",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48308799,
            "range": "± 381528",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688791,
            "range": "± 81121",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2645936,
            "range": "± 2856",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2600648,
            "range": "± 2146",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "d3a9ff36f6b976c857bee3cec778417bcae7b061",
          "message": "CI only publish local packages, no deps",
          "timestamp": "2021-11-03T14:23:34-07:00",
          "tree_id": "712d768cfad41db5f841c96f6c100fe62cfd4dd4",
          "url": "https://github.com/hydro-project/hydroflow/commit/d3a9ff36f6b976c857bee3cec778417bcae7b061"
        },
        "date": 1635975159323,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375251,
            "range": "± 2578",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198181480,
            "range": "± 755013",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 196598035,
            "range": "± 6151133",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 58897303,
            "range": "± 696921",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13040022,
            "range": "± 45243",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46816667,
            "range": "± 1634625",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48103337,
            "range": "± 656378",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687502,
            "range": "± 61553",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2638540,
            "range": "± 2567",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2606364,
            "range": "± 3264",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d3a9ff36f6b976c857bee3cec778417bcae7b061",
          "message": "CI only publish local packages, no deps",
          "timestamp": "2021-11-03T21:23:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d3a9ff36f6b976c857bee3cec778417bcae7b061"
        },
        "date": 1635997754004,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 447894,
            "range": "± 6719",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 209923918,
            "range": "± 587277",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 212794554,
            "range": "± 7952056",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 64542947,
            "range": "± 198363",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15487923,
            "range": "± 133107",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53409674,
            "range": "± 555476",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57422162,
            "range": "± 536566",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17540864,
            "range": "± 106125",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3116035,
            "range": "± 21482",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3075319,
            "range": "± 20946",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "9a191e21feb75d5d869c6ce039749d882ff1f35c",
          "message": "Make HandoffId distinct for parallel multigraph edges",
          "timestamp": "2021-11-04T13:18:46-07:00",
          "tree_id": "0f0bd95f1e1d6b70b761f97664851b05a510f665",
          "url": "https://github.com/hydro-project/hydroflow/commit/9a191e21feb75d5d869c6ce039749d882ff1f35c"
        },
        "date": 1636057683423,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375481,
            "range": "± 2618",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 190027635,
            "range": "± 241918",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190608338,
            "range": "± 6157315",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 59332192,
            "range": "± 601101",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14076754,
            "range": "± 150021",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46955414,
            "range": "± 82366",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50240966,
            "range": "± 687294",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689160,
            "range": "± 5618",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2627643,
            "range": "± 2673",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2606017,
            "range": "± 3248",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "56cec4343a9aaba80c688168d2407634c0211daf",
          "message": "Introduce an \"Input\" for ingressing data (#6)\n\nThis also does some minor optimization to make it less expensive to constantly\r\nschedule the same operator. Only did a thread-local version for now, just to\r\ntry to get it working, I think it's easy to extend to a version that can be\r\ncross-thread from here.",
          "timestamp": "2021-11-04T22:11:41Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/56cec4343a9aaba80c688168d2407634c0211daf"
        },
        "date": 1636084189388,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 384868,
            "range": "± 22245",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 234781974,
            "range": "± 6166925",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 253681998,
            "range": "± 8939243",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 69256807,
            "range": "± 3375021",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 18500100,
            "range": "± 1242007",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 56136576,
            "range": "± 2458359",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 64049689,
            "range": "± 4478736",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17407680,
            "range": "± 771768",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3274774,
            "range": "± 142151",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3250199,
            "range": "± 183231",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9a9bd5130025e6ab514bf0e32c30ea918b501821",
          "message": "Make Input threadsafe (optionally) (#8)",
          "timestamp": "2021-11-05T18:45:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9a9bd5130025e6ab514bf0e32c30ea918b501821"
        },
        "date": 1636170578065,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 460118,
            "range": "± 25206",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 221097015,
            "range": "± 4031206",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 218686344,
            "range": "± 2689454",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 68529531,
            "range": "± 664752",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 16783210,
            "range": "± 167546",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 55071680,
            "range": "± 267632",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60399949,
            "range": "± 589445",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17636609,
            "range": "± 10312",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3206692,
            "range": "± 8508",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3169604,
            "range": "± 12459",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9fad013dd15eb50158c920380cf8ab201062565f",
          "message": "Replace git dependency with workspace sibling\n\nFixes the build",
          "timestamp": "2021-11-08T19:06:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9fad013dd15eb50158c920380cf8ab201062565f"
        },
        "date": 1636408631387,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 380001,
            "range": "± 31321",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206547592,
            "range": "± 4620497",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 287474234,
            "range": "± 16809411",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 63467713,
            "range": "± 3274879",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 19304049,
            "range": "± 835699",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 58564919,
            "range": "± 2744930",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 62957075,
            "range": "± 4177533",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17928029,
            "range": "± 919857",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3275212,
            "range": "± 98965",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3238454,
            "range": "± 161918",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9fad013dd15eb50158c920380cf8ab201062565f",
          "message": "Replace git dependency with workspace sibling\n\nFixes the build",
          "timestamp": "2021-11-08T19:06:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9fad013dd15eb50158c920380cf8ab201062565f"
        },
        "date": 1636429738054,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 351336,
            "range": "± 12361",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199442165,
            "range": "± 7643605",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 281181228,
            "range": "± 20651443",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 58589686,
            "range": "± 2684464",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 18682810,
            "range": "± 948699",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 55494682,
            "range": "± 2532703",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57400085,
            "range": "± 3856721",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17505465,
            "range": "± 827825",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2995559,
            "range": "± 119419",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3038978,
            "range": "± 139344",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "f547e90838baf935e57d91bea936dd9bfa48d75d",
          "message": "Use slightly more efficient implementation of Once [ci-bench]",
          "timestamp": "2021-11-09T16:50:29-08:00",
          "tree_id": "a86f19ae77297468dfb264537b5a83a9e8380470",
          "url": "https://github.com/hydro-project/hydroflow/commit/f547e90838baf935e57d91bea936dd9bfa48d75d"
        },
        "date": 1636506012033,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 364215,
            "range": "± 23435",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 232056788,
            "range": "± 7607168",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 247080725,
            "range": "± 9422736",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 67923904,
            "range": "± 3865799",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 16295963,
            "range": "± 949107",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53846196,
            "range": "± 2969828",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59611282,
            "range": "± 3168548",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16707325,
            "range": "± 1156284",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3072453,
            "range": "± 137700",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3254752,
            "range": "± 184070",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f547e90838baf935e57d91bea936dd9bfa48d75d",
          "message": "Use slightly more efficient implementation of Once [ci-bench]",
          "timestamp": "2021-11-10T00:34:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f547e90838baf935e57d91bea936dd9bfa48d75d"
        },
        "date": 1636516141871,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 352363,
            "range": "± 17111",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 192098408,
            "range": "± 6084678",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 240505903,
            "range": "± 18983000",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 58219275,
            "range": "± 2241822",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 16826707,
            "range": "± 873721",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53133057,
            "range": "± 1852578",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56919657,
            "range": "± 3402348",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16976342,
            "range": "± 895038",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2998460,
            "range": "± 113413",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3055243,
            "range": "± 100528",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "committer": {
            "email": "mingwei.samuel@gmail.com",
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel"
          },
          "distinct": true,
          "id": "9ba809bd6b1d48bff1136bc14833712395eb9d36",
          "message": "Revert \"Use slightly more efficient implementation of Once [ci-bench]\"\n\nThis reverts commit f547e90838baf935e57d91bea936dd9bfa48d75d.",
          "timestamp": "2021-11-10T14:57:32-08:00",
          "tree_id": "a9235c8d07e792d5a0d1b2b55ea4204800d17d3b",
          "url": "https://github.com/hydro-project/hydroflow/commit/9ba809bd6b1d48bff1136bc14833712395eb9d36"
        },
        "date": 1636585603490,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375616,
            "range": "± 2727",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200402310,
            "range": "± 438035",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 195010937,
            "range": "± 1833938",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 58760929,
            "range": "± 294352",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13934138,
            "range": "± 44386",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46517454,
            "range": "± 99613",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47347617,
            "range": "± 537838",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685920,
            "range": "± 9436",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2627347,
            "range": "± 2577",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2585773,
            "range": "± 2446",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6bbbaf200f765973573f1df1b92462b57f184248",
          "message": "Make Handoff trait take shared &self",
          "timestamp": "2021-11-10T22:52:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6bbbaf200f765973573f1df1b92462b57f184248"
        },
        "date": 1636602489251,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376040,
            "range": "± 2758",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206701990,
            "range": "± 996094",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 204793616,
            "range": "± 1927904",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 70167446,
            "range": "± 360901",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12840298,
            "range": "± 61030",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 49684445,
            "range": "± 265018",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 62387941,
            "range": "± 796548",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14706223,
            "range": "± 9326",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2779590,
            "range": "± 22665",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2667410,
            "range": "± 13822",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d40c1b8789630c987f7f7605ddd21421eaf9aaee",
          "message": "Replace concrete Subgraph types with FnMut()\n\nAlso removes a few more unneccesary `&mut`s",
          "timestamp": "2021-11-11T18:33:41Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d40c1b8789630c987f7f7605ddd21421eaf9aaee"
        },
        "date": 1636688876722,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331951,
            "range": "± 2772",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187530209,
            "range": "± 3913182",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190190897,
            "range": "± 9033812",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/teer",
            "value": 63848309,
            "range": "± 670046",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12493760,
            "range": "± 54563",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42918733,
            "range": "± 2402235",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57405632,
            "range": "± 969137",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12976639,
            "range": "± 5364",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2736909,
            "range": "± 53908",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2709776,
            "range": "± 25708",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dd3b77425f8511a80ca62672c552aa8a49ce7b90",
          "message": "Change handoffs to be owned by Hydroflow, use dyn Any casts\n\nNotes\n- TeeingHandoff is commented out (TODO).\n- Variadic generics require GATs now due to reference in args.",
          "timestamp": "2021-11-11T18:04:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3b77425f8511a80ca62672c552aa8a49ce7b90"
        },
        "date": 1636702293332,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 383527,
            "range": "± 20231",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206450818,
            "range": "± 6322456",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 260838034,
            "range": "± 17830302",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11615771,
            "range": "± 746730",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 54361126,
            "range": "± 4185367",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51314888,
            "range": "± 5182038",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18013484,
            "range": "± 1001173",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3210575,
            "range": "± 209622",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3175314,
            "range": "± 196468",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dd3b77425f8511a80ca62672c552aa8a49ce7b90",
          "message": "Change handoffs to be owned by Hydroflow, use dyn Any casts\n\nNotes\n- TeeingHandoff is commented out (TODO).\n- Variadic generics require GATs now due to reference in args.",
          "timestamp": "2021-11-11T18:04:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3b77425f8511a80ca62672c552aa8a49ce7b90"
        },
        "date": 1636704816075,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375929,
            "range": "± 2499",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199188480,
            "range": "± 926167",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 189267466,
            "range": "± 3562338",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8350521,
            "range": "± 46447",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44975973,
            "range": "± 179244",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50270554,
            "range": "± 1821803",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14694104,
            "range": "± 7593",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2591817,
            "range": "± 3798",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2563773,
            "range": "± 4248",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1c70cf861b20b88ec648f968cc7ca1581bd20c9c",
          "message": "Allow creating an input operator from an existing channel (#12)",
          "timestamp": "2021-11-12T17:54:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1c70cf861b20b88ec648f968cc7ca1581bd20c9c"
        },
        "date": 1636775262179,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331945,
            "range": "± 1584",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 192510714,
            "range": "± 1315232",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 179502770,
            "range": "± 4101859",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7597785,
            "range": "± 35278",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44619637,
            "range": "± 217931",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47607149,
            "range": "± 387945",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12964236,
            "range": "± 4633",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2666219,
            "range": "± 3872",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2322945,
            "range": "± 6047",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1c70cf861b20b88ec648f968cc7ca1581bd20c9c",
          "message": "Allow creating an input operator from an existing channel (#12)",
          "timestamp": "2021-11-12T17:54:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1c70cf861b20b88ec648f968cc7ca1581bd20c9c"
        },
        "date": 1636861629387,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 352447,
            "range": "± 25047",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 185304892,
            "range": "± 3577061",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 172014315,
            "range": "± 8339216",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 6859060,
            "range": "± 321685",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41207940,
            "range": "± 2737858",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 43390793,
            "range": "± 1412784",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 13831057,
            "range": "± 554135",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2493489,
            "range": "± 113105",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2503476,
            "range": "± 128010",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1c70cf861b20b88ec648f968cc7ca1581bd20c9c",
          "message": "Allow creating an input operator from an existing channel (#12)",
          "timestamp": "2021-11-12T17:54:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1c70cf861b20b88ec648f968cc7ca1581bd20c9c"
        },
        "date": 1636948069228,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376248,
            "range": "± 2839",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204169217,
            "range": "± 5019175",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 194724989,
            "range": "± 4021654",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7606413,
            "range": "± 95258",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46182577,
            "range": "± 626898",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61716236,
            "range": "± 1405424",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14710342,
            "range": "± 6971",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2708421,
            "range": "± 34083",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2598789,
            "range": "± 20572",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "489773675ca56c4edfa6acd21135c39490c0fcc9",
          "message": "Rename OpId to SubgraphId and update variables for consistent terminology",
          "timestamp": "2021-11-12T18:03:18Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/489773675ca56c4edfa6acd21135c39490c0fcc9"
        },
        "date": 1637034469862,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371837,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196784561,
            "range": "± 707817",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 185511814,
            "range": "± 2107882",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8095346,
            "range": "± 18428",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43895649,
            "range": "± 273367",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48000436,
            "range": "± 328335",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681458,
            "range": "± 5730",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2612215,
            "range": "± 3253",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2562330,
            "range": "± 13986",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "489773675ca56c4edfa6acd21135c39490c0fcc9",
          "message": "Rename OpId to SubgraphId and update variables for consistent terminology",
          "timestamp": "2021-11-12T18:03:18Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/489773675ca56c4edfa6acd21135c39490c0fcc9"
        },
        "date": 1637120970348,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 263522,
            "range": "± 3290",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 163745155,
            "range": "± 932586",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 152462340,
            "range": "± 7380765",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8168600,
            "range": "± 25772",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40022327,
            "range": "± 269349",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57419381,
            "range": "± 361115",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18221630,
            "range": "± 14533",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2847485,
            "range": "± 3263",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2777866,
            "range": "± 2155",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9db3b81b191fde45dac4c0641f7e660a7f7e9af9",
          "message": "Remove handoff-style state connecting, instead use StateHandle as pointers with Context<'_>",
          "timestamp": "2021-11-17T02:41:44Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9db3b81b191fde45dac4c0641f7e660a7f7e9af9"
        },
        "date": 1637207345562,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 387224,
            "range": "± 20019",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 208972639,
            "range": "± 4367876",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 259337132,
            "range": "± 27228973",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11854548,
            "range": "± 446966",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53017709,
            "range": "± 1923706",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59273694,
            "range": "± 2673256",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18136141,
            "range": "± 702556",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3222574,
            "range": "± 193910",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3228859,
            "range": "± 132644",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9db3b81b191fde45dac4c0641f7e660a7f7e9af9",
          "message": "Remove handoff-style state connecting, instead use StateHandle as pointers with Context<'_>",
          "timestamp": "2021-11-17T02:41:44Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9db3b81b191fde45dac4c0641f7e660a7f7e9af9"
        },
        "date": 1637256796213,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375293,
            "range": "± 3587",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188847485,
            "range": "± 528448",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 174303655,
            "range": "± 7571912",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8687136,
            "range": "± 34179",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44347634,
            "range": "± 168199",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49848359,
            "range": "± 932518",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14690997,
            "range": "± 37543",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2613015,
            "range": "± 5765",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2597745,
            "range": "± 1779",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9db3b81b191fde45dac4c0641f7e660a7f7e9af9",
          "message": "Remove handoff-style state connecting, instead use StateHandle as pointers with Context<'_>",
          "timestamp": "2021-11-17T02:41:44Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9db3b81b191fde45dac4c0641f7e660a7f7e9af9"
        },
        "date": 1637293666387,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374929,
            "range": "± 2540",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198874117,
            "range": "± 461393",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 185884990,
            "range": "± 1630015",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8286342,
            "range": "± 77113",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44181432,
            "range": "± 140655",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48102249,
            "range": "± 327284",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683574,
            "range": "± 8405",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2605602,
            "range": "± 2417",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2570690,
            "range": "± 2216",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "246d437f15437ff258ae835fa49b922d82892c63",
          "message": "Add distributed Covid tracing app (#15)\n\nThis is a very rough first take of a Hydroflow app that crosses network\r\nboundaries. Lots of stuff is bad here: the abstractions are wonky, the encoding\r\nallocates too much, the message-passing is fragile, but I wanted to get this\r\nmerged so that stuff could be improved incrementally as we figure out what\r\nactually good abstractions are here.",
          "timestamp": "2021-11-19T20:21:23Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/246d437f15437ff258ae835fa49b922d82892c63"
        },
        "date": 1637380030902,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375663,
            "range": "± 2452",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 189310015,
            "range": "± 726434",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 164231741,
            "range": "± 1214703",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8273995,
            "range": "± 37284",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39690260,
            "range": "± 361739",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48005362,
            "range": "± 168746",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12985684,
            "range": "± 809194",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2635228,
            "range": "± 2064",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2346798,
            "range": "± 2933",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "246d437f15437ff258ae835fa49b922d82892c63",
          "message": "Add distributed Covid tracing app (#15)\n\nThis is a very rough first take of a Hydroflow app that crosses network\r\nboundaries. Lots of stuff is bad here: the abstractions are wonky, the encoding\r\nallocates too much, the message-passing is fragile, but I wanted to get this\r\nmerged so that stuff could be improved incrementally as we figure out what\r\nactually good abstractions are here.",
          "timestamp": "2021-11-19T20:21:23Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/246d437f15437ff258ae835fa49b922d82892c63"
        },
        "date": 1637466467317,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370561,
            "range": "± 2499",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198948484,
            "range": "± 538365",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 188135007,
            "range": "± 4141699",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8732330,
            "range": "± 27062",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44094370,
            "range": "± 294364",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48837023,
            "range": "± 667986",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685785,
            "range": "± 7819",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2623496,
            "range": "± 2216",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2621760,
            "range": "± 5129",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "246d437f15437ff258ae835fa49b922d82892c63",
          "message": "Add distributed Covid tracing app (#15)\n\nThis is a very rough first take of a Hydroflow app that crosses network\r\nboundaries. Lots of stuff is bad here: the abstractions are wonky, the encoding\r\nallocates too much, the message-passing is fragile, but I wanted to get this\r\nmerged so that stuff could be improved incrementally as we figure out what\r\nactually good abstractions are here.",
          "timestamp": "2021-11-19T20:21:23Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/246d437f15437ff258ae835fa49b922d82892c63"
        },
        "date": 1637552844802,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261075,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165306173,
            "range": "± 1149124",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 155049983,
            "range": "± 1671627",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8339029,
            "range": "± 39615",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39695795,
            "range": "± 160958",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57339389,
            "range": "± 60651",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18213134,
            "range": "± 21975",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2853959,
            "range": "± 14016",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2832728,
            "range": "± 2963",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f4960876e0a18ba484baef6192a2c0dcdf5d5224",
          "message": "Extract out turning a TcpStream into operators (#16)",
          "timestamp": "2021-11-22T23:18:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4960876e0a18ba484baef6192a2c0dcdf5d5224"
        },
        "date": 1637639266106,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261659,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168705712,
            "range": "± 794282",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 155907366,
            "range": "± 3816694",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8125513,
            "range": "± 57694",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39439741,
            "range": "± 380989",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57822368,
            "range": "± 1133367",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18230254,
            "range": "± 11297",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2816983,
            "range": "± 27088",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2849145,
            "range": "± 3752",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f4960876e0a18ba484baef6192a2c0dcdf5d5224",
          "message": "Extract out turning a TcpStream into operators (#16)",
          "timestamp": "2021-11-22T23:18:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4960876e0a18ba484baef6192a2c0dcdf5d5224"
        },
        "date": 1637725665993,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 268051,
            "range": "± 5324",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 163496392,
            "range": "± 785666",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 153211315,
            "range": "± 4759245",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8154498,
            "range": "± 22561",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39772983,
            "range": "± 258956",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57334961,
            "range": "± 154209",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18212036,
            "range": "± 15220",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2811340,
            "range": "± 3563",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2843232,
            "range": "± 1801",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce",
          "message": "Eliminate some copies and batch networks sends (#17)\n\nThis implementation still has some extra copies, and doesn't re-use buffers\r\nappropriately. In addition it still just encodes in JSON since that makes it a\r\nbit easier to debug over the wire.",
          "timestamp": "2021-11-24T19:03:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce"
        },
        "date": 1637812094728,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451044,
            "range": "± 2949",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 212986225,
            "range": "± 6810371",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 201366036,
            "range": "± 2723670",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10046524,
            "range": "± 198329",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 50489677,
            "range": "± 1208445",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55194918,
            "range": "± 1157947",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16702414,
            "range": "± 358988",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3076872,
            "range": "± 45351",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3042095,
            "range": "± 53593",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce",
          "message": "Eliminate some copies and batch networks sends (#17)\n\nThis implementation still has some extra copies, and doesn't re-use buffers\r\nappropriately. In addition it still just encodes in JSON since that makes it a\r\nbit easier to debug over the wire.",
          "timestamp": "2021-11-24T19:03:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce"
        },
        "date": 1637898518637,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371055,
            "range": "± 23432",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 227360458,
            "range": "± 10020985",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 223822367,
            "range": "± 11358765",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9834093,
            "range": "± 526002",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53580852,
            "range": "± 4287092",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59741609,
            "range": "± 3098348",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17050664,
            "range": "± 1034140",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2900744,
            "range": "± 184001",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3015487,
            "range": "± 197265",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce",
          "message": "Eliminate some copies and batch networks sends (#17)\n\nThis implementation still has some extra copies, and doesn't re-use buffers\r\nappropriately. In addition it still just encodes in JSON since that makes it a\r\nbit easier to debug over the wire.",
          "timestamp": "2021-11-24T19:03:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce"
        },
        "date": 1637984881363,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375797,
            "range": "± 2706",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199768892,
            "range": "± 1218122",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190131419,
            "range": "± 1597856",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8781746,
            "range": "± 52200",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44456584,
            "range": "± 148062",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49132079,
            "range": "± 731890",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687325,
            "range": "± 7820",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2604886,
            "range": "± 4995",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2617537,
            "range": "± 1784",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce",
          "message": "Eliminate some copies and batch networks sends (#17)\n\nThis implementation still has some extra copies, and doesn't re-use buffers\r\nappropriately. In addition it still just encodes in JSON since that makes it a\r\nbit easier to debug over the wire.",
          "timestamp": "2021-11-24T19:03:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce"
        },
        "date": 1638071349094,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 389912,
            "range": "± 24613",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 230542851,
            "range": "± 5124465",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 231730129,
            "range": "± 5075936",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10582870,
            "range": "± 455772",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 56217338,
            "range": "± 2302246",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 65009047,
            "range": "± 4054486",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18637329,
            "range": "± 646383",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3314413,
            "range": "± 133268",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3332785,
            "range": "± 138965",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce",
          "message": "Eliminate some copies and batch networks sends (#17)\n\nThis implementation still has some extra copies, and doesn't re-use buffers\r\nappropriately. In addition it still just encodes in JSON since that makes it a\r\nbit easier to debug over the wire.",
          "timestamp": "2021-11-24T19:03:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8fc5a4d0b4e6e01d4b83e6708425a44ebc4272ce"
        },
        "date": 1638157679898,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 431212,
            "range": "± 13100",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 209370191,
            "range": "± 1839859",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 197786555,
            "range": "± 6160261",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10071903,
            "range": "± 229402",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 49744210,
            "range": "± 1449423",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55472159,
            "range": "± 1606688",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17219340,
            "range": "± 451383",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2976701,
            "range": "± 64668",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3095154,
            "range": "± 38205",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "418803b7c86a0ebe0cf29ebec47c99108afc1cee",
          "message": "Push some of the networking stuff into a library (#18)",
          "timestamp": "2021-11-29T19:26:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/418803b7c86a0ebe0cf29ebec47c99108afc1cee"
        },
        "date": 1638243999801,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375914,
            "range": "± 2555",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200185012,
            "range": "± 1016955",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 193502972,
            "range": "± 4090012",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7749254,
            "range": "± 104958",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46988653,
            "range": "± 1704957",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51457394,
            "range": "± 1813379",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14693097,
            "range": "± 5983",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2853352,
            "range": "± 19643",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2817765,
            "range": "± 126631",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "418803b7c86a0ebe0cf29ebec47c99108afc1cee",
          "message": "Push some of the networking stuff into a library (#18)",
          "timestamp": "2021-11-29T19:26:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/418803b7c86a0ebe0cf29ebec47c99108afc1cee"
        },
        "date": 1638330538792,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 350368,
            "range": "± 34047",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 226195797,
            "range": "± 7245519",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 234500229,
            "range": "± 12110701",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11063096,
            "range": "± 656248",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 51484297,
            "range": "± 2000677",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60235131,
            "range": "± 2903781",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16653728,
            "range": "± 883507",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3408084,
            "range": "± 193714",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3436748,
            "range": "± 242408",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "852371ff6bc52f7e64ce81879bccab2ee9fa381e",
          "message": "Update all hydroflow subgraphs to take a Context<'_>",
          "timestamp": "2021-12-01T23:37:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/852371ff6bc52f7e64ce81879bccab2ee9fa381e"
        },
        "date": 1638416871160,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375605,
            "range": "± 2458",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195847048,
            "range": "± 1084830",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 187100172,
            "range": "± 1546327",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8109639,
            "range": "± 87072",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47605618,
            "range": "± 1207616",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48241380,
            "range": "± 1820697",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681492,
            "range": "± 10023",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2826169,
            "range": "± 66589",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2805103,
            "range": "± 21394",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f29ae98473dddda1c3225a91b8e87da8c4b1bab",
          "message": "Remove tokio runtime, use Context<'_>'s Waker for networking",
          "timestamp": "2021-12-02T18:59:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f29ae98473dddda1c3225a91b8e87da8c4b1bab"
        },
        "date": 1638503270909,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370716,
            "range": "± 2526",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196416287,
            "range": "± 1015327",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 191462516,
            "range": "± 3139029",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8828252,
            "range": "± 59723",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43723155,
            "range": "± 191576",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47692036,
            "range": "± 1234178",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683321,
            "range": "± 7563",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2737719,
            "range": "± 8969",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2666421,
            "range": "± 7709",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f29ae98473dddda1c3225a91b8e87da8c4b1bab",
          "message": "Remove tokio runtime, use Context<'_>'s Waker for networking",
          "timestamp": "2021-12-02T18:59:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f29ae98473dddda1c3225a91b8e87da8c4b1bab"
        },
        "date": 1638589686897,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375759,
            "range": "± 2447",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200132678,
            "range": "± 698269",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190787915,
            "range": "± 1567888",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8462963,
            "range": "± 69397",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44441597,
            "range": "± 212190",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50874085,
            "range": "± 1101752",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14696164,
            "range": "± 23455",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2626425,
            "range": "± 19459",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2624652,
            "range": "± 2537",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f29ae98473dddda1c3225a91b8e87da8c4b1bab",
          "message": "Remove tokio runtime, use Context<'_>'s Waker for networking",
          "timestamp": "2021-12-02T18:59:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f29ae98473dddda1c3225a91b8e87da8c4b1bab"
        },
        "date": 1638676083306,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375630,
            "range": "± 2598",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195769308,
            "range": "± 377105",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 185452953,
            "range": "± 5097526",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7422661,
            "range": "± 14062",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43526507,
            "range": "± 164210",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48057285,
            "range": "± 274783",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685373,
            "range": "± 9183",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2584949,
            "range": "± 2361",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2581566,
            "range": "± 14250",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f29ae98473dddda1c3225a91b8e87da8c4b1bab",
          "message": "Remove tokio runtime, use Context<'_>'s Waker for networking",
          "timestamp": "2021-12-02T18:59:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f29ae98473dddda1c3225a91b8e87da8c4b1bab"
        },
        "date": 1638762554370,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 377465,
            "range": "± 18575",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 231106014,
            "range": "± 7600119",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 235716872,
            "range": "± 5305434",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10559606,
            "range": "± 942114",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 52272377,
            "range": "± 2773443",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 62826852,
            "range": "± 3601334",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17733712,
            "range": "± 694898",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3153109,
            "range": "± 192000",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3177103,
            "range": "± 139368",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f29ae98473dddda1c3225a91b8e87da8c4b1bab",
          "message": "Remove tokio runtime, use Context<'_>'s Waker for networking",
          "timestamp": "2021-12-02T18:59:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f29ae98473dddda1c3225a91b8e87da8c4b1bab"
        },
        "date": 1638848894422,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 271254,
            "range": "± 5656",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 164273297,
            "range": "± 1452248",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 154714946,
            "range": "± 4303488",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8675994,
            "range": "± 33155",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39701518,
            "range": "± 247532",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57564326,
            "range": "± 126514",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18216630,
            "range": "± 18347",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2820718,
            "range": "± 3237",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2804751,
            "range": "± 1907",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5dab68b90a4892120e34fddebdec4fb094adb523",
          "message": "Add a basic implementation of relational algebra (#23)\n\nThis is a WIP implementation of relational algebra that runs in Hydroflow.\r\n\r\nRight now it only uses scheduled components and is only at runtime, no codegen,\r\njust for simplicity. It's also not how I would design a non-proof-of-concept IR\r\nbut I think for our purposes for now it will be fine.",
          "timestamp": "2021-12-07T18:48:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5dab68b90a4892120e34fddebdec4fb094adb523"
        },
        "date": 1638935320787,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 436924,
            "range": "± 6652",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 209765941,
            "range": "± 1597314",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 182947538,
            "range": "± 4466322",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9590119,
            "range": "± 116987",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 50115542,
            "range": "± 517830",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55392749,
            "range": "± 710221",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17064224,
            "range": "± 202164",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3049236,
            "range": "± 32536",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2971586,
            "range": "± 28395",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "964de20560c5bc856ad6c3e5216fe58f309e0a17",
          "message": "Add a simple codegen path for relational algebra (#24)\n\nThis currenly only handles some very simple patterns, and doesn't invoke\r\nscheduled components at all, but this is the general pattern I was thinking for\r\nnow. Comments on the overall design welcome, you can see examples of the\r\ngenerated code in testdata/compile/compile.\r\n\r\nWhat's more, I think this PR will probably fail on CI since it shells out to\r\nrustfmt to format the generated code. Suggestions welcome for how to fix, I\r\nmight just make it skip running the test (probably with a warning) if rustfmt\r\ndoes not just work.\r\n\r\nNext step is to introduce a let binding layer to the language, to force the\r\ncodegen to use scheduled components/invoke Hydroflow.",
          "timestamp": "2021-12-08T20:48:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/964de20560c5bc856ad6c3e5216fe58f309e0a17"
        },
        "date": 1639021678572,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 274542,
            "range": "± 6763",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 164693135,
            "range": "± 572441",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 154730601,
            "range": "± 3221008",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8067106,
            "range": "± 15296",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40076250,
            "range": "± 188232",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57638332,
            "range": "± 96551",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209264,
            "range": "± 97726",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2853006,
            "range": "± 4880",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2833171,
            "range": "± 3034",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "359b1625a19566ef8eff5cb6f6d742ebb99ba8e3",
          "message": "rename tlt! macro to tt!",
          "timestamp": "2021-12-09T23:09:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/359b1625a19566ef8eff5cb6f6d742ebb99ba8e3"
        },
        "date": 1639108109805,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 420363,
            "range": "± 11418",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206168820,
            "range": "± 2325578",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 192613834,
            "range": "± 2420509",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9873845,
            "range": "± 221126",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 48778454,
            "range": "± 1117651",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 53490751,
            "range": "± 1794506",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17992819,
            "range": "± 590697",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2882444,
            "range": "± 106610",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2823713,
            "range": "± 83349",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "359b1625a19566ef8eff5cb6f6d742ebb99ba8e3",
          "message": "rename tlt! macro to tt!",
          "timestamp": "2021-12-09T23:09:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/359b1625a19566ef8eff5cb6f6d742ebb99ba8e3"
        },
        "date": 1639194468243,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 305583,
            "range": "± 8670",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165447830,
            "range": "± 848865",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 153692587,
            "range": "± 1464137",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8793940,
            "range": "± 37854",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39656519,
            "range": "± 567546",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57711445,
            "range": "± 198060",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15865402,
            "range": "± 214265",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2871156,
            "range": "± 2737",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2783932,
            "range": "± 15575",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "359b1625a19566ef8eff5cb6f6d742ebb99ba8e3",
          "message": "rename tlt! macro to tt!",
          "timestamp": "2021-12-09T23:09:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/359b1625a19566ef8eff5cb6f6d742ebb99ba8e3"
        },
        "date": 1639280877104,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375972,
            "range": "± 2789",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199025960,
            "range": "± 506270",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190597752,
            "range": "± 3776681",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7884275,
            "range": "± 84289",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44527715,
            "range": "± 75246",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 53781797,
            "range": "± 1073934",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15920305,
            "range": "± 6695",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2657553,
            "range": "± 25929",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2600260,
            "range": "± 2893",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "359b1625a19566ef8eff5cb6f6d742ebb99ba8e3",
          "message": "rename tlt! macro to tt!",
          "timestamp": "2021-12-09T23:09:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/359b1625a19566ef8eff5cb6f6d742ebb99ba8e3"
        },
        "date": 1639367278656,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371502,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187774883,
            "range": "± 1071812",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 182704377,
            "range": "± 1410114",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7433829,
            "range": "± 8825",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44637681,
            "range": "± 654195",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48496890,
            "range": "± 440369",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684210,
            "range": "± 8848",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2655807,
            "range": "± 3497",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2585677,
            "range": "± 2099",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "359b1625a19566ef8eff5cb6f6d742ebb99ba8e3",
          "message": "rename tlt! macro to tt!",
          "timestamp": "2021-12-09T23:09:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/359b1625a19566ef8eff5cb6f6d742ebb99ba8e3"
        },
        "date": 1639453709405,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 400444,
            "range": "± 26233",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 214437420,
            "range": "± 2239560",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 183097218,
            "range": "± 5396483",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9471417,
            "range": "± 526492",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 52140466,
            "range": "± 1119551",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57808201,
            "range": "± 1250787",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17137119,
            "range": "± 502923",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3083901,
            "range": "± 81227",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3033512,
            "range": "± 30370",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cd8ecacaa1d1c09bb831f3d10f7c3d2c240b6b88",
          "message": "Make CI check all targets",
          "timestamp": "2021-12-15T23:25:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cd8ecacaa1d1c09bb831f3d10f7c3d2c240b6b88"
        },
        "date": 1639626391513,
        "tool": "cargo",
        "benches": [
          {
            "name": "fan_in/hydroflow",
            "value": 163048551,
            "range": "± 781979",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 154031008,
            "range": "± 8721194",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8913618,
            "range": "± 48888",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39412455,
            "range": "± 169398",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57238715,
            "range": "± 196488",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2987352,
            "range": "± 3954",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1639712888009,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 458649,
            "range": "± 32895",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216824716,
            "range": "± 702141",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 203317702,
            "range": "± 1711404",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10467668,
            "range": "± 62418",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 52058253,
            "range": "± 445771",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56801326,
            "range": "± 899160",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17585934,
            "range": "± 108686",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3184920,
            "range": "± 93708",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3137549,
            "range": "± 27457",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1639799345380,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374950,
            "range": "± 27492",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 230193276,
            "range": "± 6246239",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 233975809,
            "range": "± 7391161",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11491312,
            "range": "± 637696",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53236986,
            "range": "± 6425333",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60410304,
            "range": "± 2809571",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18489033,
            "range": "± 974881",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3256461,
            "range": "± 120683",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3151976,
            "range": "± 198528",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1639885635221,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375643,
            "range": "± 2743",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 197440404,
            "range": "± 756147",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 188716200,
            "range": "± 1949947",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8281114,
            "range": "± 34809",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38138201,
            "range": "± 136361",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41741567,
            "range": "± 406394",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12865883,
            "range": "± 77933",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2328677,
            "range": "± 19732",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2308849,
            "range": "± 12588",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1639972083181,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376271,
            "range": "± 3077",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 203646782,
            "range": "± 798798",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 195004217,
            "range": "± 1297055",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8340965,
            "range": "± 57374",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47567093,
            "range": "± 542751",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61987555,
            "range": "± 1491777",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14708454,
            "range": "± 7796",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2680154,
            "range": "± 20199",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2638028,
            "range": "± 17891",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640058523456,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 368976,
            "range": "± 31444",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 201457226,
            "range": "± 6552523",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 244493823,
            "range": "± 23030730",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10686360,
            "range": "± 799477",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 49971878,
            "range": "± 2282964",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55463860,
            "range": "± 2908218",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17393785,
            "range": "± 985921",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3016128,
            "range": "± 174394",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3037168,
            "range": "± 226657",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640144914084,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 444226,
            "range": "± 5956",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 227108869,
            "range": "± 6494045",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 205702446,
            "range": "± 3999157",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9970405,
            "range": "± 637352",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 54782200,
            "range": "± 5768228",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57363415,
            "range": "± 1341981",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17426818,
            "range": "± 281440",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3299664,
            "range": "± 203905",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3104371,
            "range": "± 7396",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640231235230,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261813,
            "range": "± 4541",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165623821,
            "range": "± 752410",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 157603057,
            "range": "± 3808357",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8856234,
            "range": "± 48963",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42333566,
            "range": "± 218511",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57610836,
            "range": "± 57203",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18205962,
            "range": "± 70832",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2868879,
            "range": "± 6593",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2857377,
            "range": "± 3033",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640317698843,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375634,
            "range": "± 2551",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200699106,
            "range": "± 646386",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 186403691,
            "range": "± 4557194",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8619629,
            "range": "± 32878",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43622730,
            "range": "± 211807",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48136696,
            "range": "± 697384",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681598,
            "range": "± 10620",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2708719,
            "range": "± 21176",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2704463,
            "range": "± 5603",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640404057195,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374719,
            "range": "± 2919",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 197564875,
            "range": "± 742014",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 187489247,
            "range": "± 2331442",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8766042,
            "range": "± 33749",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43842076,
            "range": "± 198321",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48462882,
            "range": "± 613906",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691140,
            "range": "± 83306",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2623320,
            "range": "± 2936",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2588253,
            "range": "± 1600",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640490458830,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376314,
            "range": "± 2751",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195531616,
            "range": "± 1017383",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 190623158,
            "range": "± 2890171",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8905521,
            "range": "± 35343",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44862433,
            "range": "± 395998",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58718855,
            "range": "± 2357002",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14715941,
            "range": "± 47647",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2739361,
            "range": "± 13996",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2683811,
            "range": "± 11478",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640663235009,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375698,
            "range": "± 3866",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187413715,
            "range": "± 588748",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 186524382,
            "range": "± 2333624",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7873912,
            "range": "± 67751",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42843366,
            "range": "± 158218",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48548232,
            "range": "± 406677",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685675,
            "range": "± 5799",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2656508,
            "range": "± 4692",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2626898,
            "range": "± 3717",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640749684655,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 357171,
            "range": "± 13331",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 228688408,
            "range": "± 4711338",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 221049139,
            "range": "± 8069776",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9961672,
            "range": "± 501197",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 53160097,
            "range": "± 4066816",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 63899845,
            "range": "± 3597301",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16992507,
            "range": "± 767976",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3156317,
            "range": "± 171843",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3121582,
            "range": "± 140423",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640836077428,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 364027,
            "range": "± 19632",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 223633631,
            "range": "± 5951576",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 215013227,
            "range": "± 6517257",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9929310,
            "range": "± 432282",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 48210242,
            "range": "± 3307140",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60331076,
            "range": "± 2936425",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16668950,
            "range": "± 846924",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3253795,
            "range": "± 214248",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3268029,
            "range": "± 216241",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1640922453454,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374726,
            "range": "± 2623",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196665747,
            "range": "± 922218",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 186702977,
            "range": "± 2009290",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8201553,
            "range": "± 34057",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43860554,
            "range": "± 107766",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47552308,
            "range": "± 302774",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14674882,
            "range": "± 10966",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2680188,
            "range": "± 15219",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2673355,
            "range": "± 2233",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1641008865443,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375600,
            "range": "± 2534",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 196591591,
            "range": "± 560313",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 183316135,
            "range": "± 10617277",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7849610,
            "range": "± 16465",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43215799,
            "range": "± 97035",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47551373,
            "range": "± 93519",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679726,
            "range": "± 12323",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2689400,
            "range": "± 2809",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2669694,
            "range": "± 1967",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1641095260703,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 372203,
            "range": "± 2514",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 205105312,
            "range": "± 2760359",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 193531951,
            "range": "± 3433045",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8598279,
            "range": "± 16631",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46989347,
            "range": "± 236147",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 65259432,
            "range": "± 797581",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14701922,
            "range": "± 35416",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2935833,
            "range": "± 26267",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2899143,
            "range": "± 23078",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc48b7161821ac7945792b73f1a2427e6faa434c",
          "message": "Add a real networked test and polish the API a bit (#32)\n\nWe want to start moving towards a usable \"clustering\" API, so this is a first\r\nstep there to make networking a little simpler/cleaner, and pull some stuff out\r\nof the Hydroflow struct for now.",
          "timestamp": "2021-12-17T01:21:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fc48b7161821ac7945792b73f1a2427e6faa434c"
        },
        "date": 1641181701512,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451005,
            "range": "± 2781",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 219793375,
            "range": "± 1095762",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 203500520,
            "range": "± 4080130",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10584477,
            "range": "± 43474",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 51380777,
            "range": "± 585756",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57470408,
            "range": "± 1007441",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17394928,
            "range": "± 203568",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3219164,
            "range": "± 43854",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3394207,
            "range": "± 57219",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "bcb63279e0980f9faa8ea0163eb66d218fa5df36",
          "message": "Fix clippy::drop_copy",
          "timestamp": "2022-01-03T18:41:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/bcb63279e0980f9faa8ea0163eb66d218fa5df36"
        },
        "date": 1641268045519,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 364837,
            "range": "± 22528",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206263083,
            "range": "± 5870570",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 188667189,
            "range": "± 7415492",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8536634,
            "range": "± 319396",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43282522,
            "range": "± 1677077",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40862107,
            "range": "± 2558725",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15301667,
            "range": "± 523716",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2954594,
            "range": "± 165151",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2766623,
            "range": "± 119007",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "efdb647f87c3402477f48ef169e32c4f2b458d58",
          "message": "cross join (#33)\n\n* cross join\r\n\r\n* fix linter errors\r\n\r\n* more linter cleanup\r\n\r\n* address clones in crossjoin\r\n\r\n* change CrossJoin to RippleJoin\r\n\r\n* satisfy linter\r\n\r\n* make clippy accept uncollapsed else if for readability",
          "timestamp": "2022-01-04T16:55:23Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/efdb647f87c3402477f48ef169e32c4f2b458d58"
        },
        "date": 1641354472324,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375638,
            "range": "± 3612",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195692727,
            "range": "± 624072",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 185424271,
            "range": "± 1500196",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9605167,
            "range": "± 34737",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43114970,
            "range": "± 71660",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47783076,
            "range": "± 120843",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14678383,
            "range": "± 11229",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2703172,
            "range": "± 2168",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2702701,
            "range": "± 1846",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1b5dcb5953221e0224f3fac9880650f0715915a8",
          "message": "Add more primitives to construct more complex graphs (#34)\n\nThis commit introduces some tools for more complex topologies and add a test\r\nthat has multiple graphs coordinating. I don't really like how much boilerplate\r\nis involved in the setup of this graph but I think it's a good starting point\r\nto figure out what these abstractions should look like.",
          "timestamp": "2022-01-06T01:16:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1b5dcb5953221e0224f3fac9880650f0715915a8"
        },
        "date": 1641440906822,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 409811,
            "range": "± 17626",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206455511,
            "range": "± 5310018",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 191696786,
            "range": "± 6061021",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8140710,
            "range": "± 282079",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 45125612,
            "range": "± 2012638",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52561301,
            "range": "± 2209727",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15472047,
            "range": "± 485907",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3118163,
            "range": "± 70672",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3114362,
            "range": "± 59736",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1b5dcb5953221e0224f3fac9880650f0715915a8",
          "message": "Add more primitives to construct more complex graphs (#34)\n\nThis commit introduces some tools for more complex topologies and add a test\r\nthat has multiple graphs coordinating. I don't really like how much boilerplate\r\nis involved in the setup of this graph but I think it's a good starting point\r\nto figure out what these abstractions should look like.",
          "timestamp": "2022-01-06T01:16:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1b5dcb5953221e0224f3fac9880650f0715915a8"
        },
        "date": 1641527302805,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 372466,
            "range": "± 2412",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 181958852,
            "range": "± 504888",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 180277682,
            "range": "± 1772509",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8384607,
            "range": "± 31236",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43718133,
            "range": "± 180210",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47898742,
            "range": "± 431185",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688948,
            "range": "± 21183",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2758292,
            "range": "± 3195",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2736813,
            "range": "± 2348",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044",
          "message": "Add an echo server test (#37)\n\nStill rough around the edges, connect_tcp in particular is not quite right, but\r\nwanted to get this out there to unblock some stuff.",
          "timestamp": "2022-01-07T19:40:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044"
        },
        "date": 1641613685512,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375601,
            "range": "± 2765",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 191076181,
            "range": "± 719595",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 176343721,
            "range": "± 3443167",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8266893,
            "range": "± 32433",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43499351,
            "range": "± 91594",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47862911,
            "range": "± 115617",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14680449,
            "range": "± 11543",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2725611,
            "range": "± 2872",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2734419,
            "range": "± 8717",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044",
          "message": "Add an echo server test (#37)\n\nStill rough around the edges, connect_tcp in particular is not quite right, but\r\nwanted to get this out there to unblock some stuff.",
          "timestamp": "2022-01-07T19:40:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044"
        },
        "date": 1641700076258,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370131,
            "range": "± 2737",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 194977724,
            "range": "± 355584",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 182709956,
            "range": "± 1315766",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8318795,
            "range": "± 20779",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42527492,
            "range": "± 67518",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47815003,
            "range": "± 246796",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14676584,
            "range": "± 123863",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2713137,
            "range": "± 3722",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2739710,
            "range": "± 1505",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044",
          "message": "Add an echo server test (#37)\n\nStill rough around the edges, connect_tcp in particular is not quite right, but\r\nwanted to get this out there to unblock some stuff.",
          "timestamp": "2022-01-07T19:40:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044"
        },
        "date": 1641786464007,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 332234,
            "range": "± 14559",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 184882311,
            "range": "± 5779060",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 169704741,
            "range": "± 6256832",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8054574,
            "range": "± 208355",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40117264,
            "range": "± 2515741",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 42593152,
            "range": "± 1334837",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12904274,
            "range": "± 345915",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2345648,
            "range": "± 63999",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2418959,
            "range": "± 57621",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044",
          "message": "Add an echo server test (#37)\n\nStill rough around the edges, connect_tcp in particular is not quite right, but\r\nwanted to get this out there to unblock some stuff.",
          "timestamp": "2022-01-07T19:40:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/03fa27ba0a4c10ac9fa8fda1f021a839f5dcb044"
        },
        "date": 1641872887220,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370520,
            "range": "± 3071",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188681811,
            "range": "± 377790",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 178101776,
            "range": "± 2552964",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9102705,
            "range": "± 39133",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43208098,
            "range": "± 240932",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47583367,
            "range": "± 285062",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681601,
            "range": "± 66792",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2604684,
            "range": "± 1627",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2711676,
            "range": "± 2488",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "760efed339e3d5a98d403a99602615f4f292d3ac",
          "message": "Clean up the TCP stuff a bit more (#39)\n\nThere's still some ergonomic issues to be worked out but I'd like to wait on\r\nthe API changes going in before trying to resolve that stuff (for example, you\r\ncan't build this into a compiled subgraph easily).",
          "timestamp": "2022-01-11T21:09:00Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/760efed339e3d5a98d403a99602615f4f292d3ac"
        },
        "date": 1641959323685,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 440933,
            "range": "± 6195",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 213229193,
            "range": "± 1485169",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 198305699,
            "range": "± 6776171",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8772130,
            "range": "± 134329",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 51471804,
            "range": "± 490076",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55804811,
            "range": "± 629955",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17133824,
            "range": "± 201717",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3121385,
            "range": "± 33979",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3294386,
            "range": "± 36194",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "760efed339e3d5a98d403a99602615f4f292d3ac",
          "message": "Clean up the TCP stuff a bit more (#39)\n\nThere's still some ergonomic issues to be worked out but I'd like to wait on\r\nthe API changes going in before trying to resolve that stuff (for example, you\r\ncan't build this into a compiled subgraph easily).",
          "timestamp": "2022-01-11T21:09:00Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/760efed339e3d5a98d403a99602615f4f292d3ac"
        },
        "date": 1642045706013,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 424699,
            "range": "± 15533",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204553775,
            "range": "± 3514296",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 191748012,
            "range": "± 6558533",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9239519,
            "range": "± 289663",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47832467,
            "range": "± 1583136",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56006654,
            "range": "± 1545422",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15989668,
            "range": "± 491448",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2893126,
            "range": "± 107129",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2943037,
            "range": "± 88838",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b57b70999014c16e99eae7d9189e9d013c0ea462",
          "message": "Switch from a bespoke format -> bincode (#40)\n\nThis commit tidies up the TCP stuff more. There's still more to be done in\r\nterms of ergonomics but I want to wait for the API change to work on that. I\r\nthink the next steps are that, and also writing a benchmark against a\r\nhand-coded server, since I think there's a lot of low-hanging fruit in terms of\r\nstuff like buffer reuse.",
          "timestamp": "2022-01-13T19:02:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b57b70999014c16e99eae7d9189e9d013c0ea462"
        },
        "date": 1642132084389,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374873,
            "range": "± 2409",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186256870,
            "range": "± 321683",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 173964726,
            "range": "± 1505282",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8408748,
            "range": "± 29041",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43352044,
            "range": "± 174087",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47817161,
            "range": "± 223095",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685200,
            "range": "± 7180",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2584157,
            "range": "± 1941",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2679121,
            "range": "± 3468",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f4ca34989ebf50a61939c3ad3d64f24daa219b29",
          "message": "fixup! CI only publish local packages, no deps",
          "timestamp": "2022-01-15T00:16:59Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4ca34989ebf50a61939c3ad3d64f24daa219b29"
        },
        "date": 1642218592897,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 379866,
            "range": "± 13588",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 201910583,
            "range": "± 6099746",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 398924150,
            "range": "± 10117440",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11557880,
            "range": "± 410736",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 51982689,
            "range": "± 1936043",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48287347,
            "range": "± 2382367",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17894014,
            "range": "± 1015597",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3188254,
            "range": "± 199829",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3294881,
            "range": "± 123896",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f4ca34989ebf50a61939c3ad3d64f24daa219b29",
          "message": "fixup! CI only publish local packages, no deps",
          "timestamp": "2022-01-15T00:16:59Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4ca34989ebf50a61939c3ad3d64f24daa219b29"
        },
        "date": 1642304888168,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375655,
            "range": "± 3700",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 197029948,
            "range": "± 491880",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 179625387,
            "range": "± 1332450",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9114767,
            "range": "± 31775",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43855646,
            "range": "± 64615",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47511414,
            "range": "± 263870",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14672039,
            "range": "± 17097",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2620199,
            "range": "± 2359",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2715788,
            "range": "± 3005",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f4ca34989ebf50a61939c3ad3d64f24daa219b29",
          "message": "fixup! CI only publish local packages, no deps",
          "timestamp": "2022-01-15T00:16:59Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4ca34989ebf50a61939c3ad3d64f24daa219b29"
        },
        "date": 1642391312993,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 372768,
            "range": "± 2606",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198652344,
            "range": "± 567086",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 189358771,
            "range": "± 1517010",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9685936,
            "range": "± 37778",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44149738,
            "range": "± 255885",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50277694,
            "range": "± 974906",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14690932,
            "range": "± 4928",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2677645,
            "range": "± 1981",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3559122,
            "range": "± 2274",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f4ca34989ebf50a61939c3ad3d64f24daa219b29",
          "message": "fixup! CI only publish local packages, no deps",
          "timestamp": "2022-01-15T00:16:59Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f4ca34989ebf50a61939c3ad3d64f24daa219b29"
        },
        "date": 1642477729770,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 333170,
            "range": "± 11745",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186836243,
            "range": "± 6680992",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 206303309,
            "range": "± 12071848",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9282164,
            "range": "± 269387",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44832045,
            "range": "± 1991171",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51312723,
            "range": "± 1850471",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15830500,
            "range": "± 669777",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2780279,
            "range": "± 154437",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2910387,
            "range": "± 114167",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4ec58f4e0b1f92b42e8a79fd268e90688ae0b1b3",
          "message": "Rewrite echo server using the surface API (#43)\n\n* Rewrite echo server using the surface API\r\n\r\nSort of minimal use of it so far since the logic here is pretty simple, but\r\nshould provide a foundation for things like address interning.\r\n\r\nWasn't sure if the wrap_{input,output} stuff aligns with your vision? LMK if\r\nthere's a cleaner/existing way to do that?\r\n\r\n* More refactoring of echo server",
          "timestamp": "2022-01-18T17:28:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/4ec58f4e0b1f92b42e8a79fd268e90688ae0b1b3"
        },
        "date": 1642564050799,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 348133,
            "range": "± 17757",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186580596,
            "range": "± 5388144",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 210812460,
            "range": "± 10969501",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10502737,
            "range": "± 455509",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 49231837,
            "range": "± 1315331",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54257550,
            "range": "± 1613204",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16495499,
            "range": "± 605630",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2914729,
            "range": "± 179439",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3062278,
            "range": "± 117268",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "480f203898652ed33018da8a6f6e42414b60a5e8",
          "message": "Add partition operator to surface API (#48)",
          "timestamp": "2022-01-20T00:27:36Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/480f203898652ed33018da8a6f6e42414b60a5e8"
        },
        "date": 1642650396238,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371001,
            "range": "± 2465",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188417837,
            "range": "± 585279",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 178338378,
            "range": "± 2821599",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8426148,
            "range": "± 27376",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44384860,
            "range": "± 397813",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48261244,
            "range": "± 283313",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683383,
            "range": "± 9813",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2630651,
            "range": "± 4688",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2693720,
            "range": "± 3160",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "89fc4cce133f4539e44dde12b919a0019ba6e925",
          "message": "chat example (#46)\n\n* working toward chat\r\n\r\n* simple chat working\r\n\r\n* rebase working toward chat\r\n\r\n* msgs sent to server\r\n\r\n* simple chat working\r\n\r\n* finish merge\r\n\r\n* wip chat\r\n\r\n* chat v0.0\r\n\r\n* unused imports\r\n\r\n* clean up chat\r\n\r\n* make clippy happy",
          "timestamp": "2022-01-21T03:04:08Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/89fc4cce133f4539e44dde12b919a0019ba6e925"
        },
        "date": 1642736808069,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375812,
            "range": "± 3027",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200252153,
            "range": "± 760226",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 189637074,
            "range": "± 2964526",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9562651,
            "range": "± 24505",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43229711,
            "range": "± 143337",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48301016,
            "range": "± 495447",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14692466,
            "range": "± 5751",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2599556,
            "range": "± 3340",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2721125,
            "range": "± 2429",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "89fc4cce133f4539e44dde12b919a0019ba6e925",
          "message": "chat example (#46)\n\n* working toward chat\r\n\r\n* simple chat working\r\n\r\n* rebase working toward chat\r\n\r\n* msgs sent to server\r\n\r\n* simple chat working\r\n\r\n* finish merge\r\n\r\n* wip chat\r\n\r\n* chat v0.0\r\n\r\n* unused imports\r\n\r\n* clean up chat\r\n\r\n* make clippy happy",
          "timestamp": "2022-01-21T03:04:08Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/89fc4cce133f4539e44dde12b919a0019ba6e925"
        },
        "date": 1642823225075,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 330335,
            "range": "± 17882",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 207614312,
            "range": "± 10100851",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 205461676,
            "range": "± 7694321",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9272032,
            "range": "± 432238",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47592396,
            "range": "± 3085616",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52655145,
            "range": "± 2672644",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15043098,
            "range": "± 917885",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2806155,
            "range": "± 115589",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2986965,
            "range": "± 134030",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "89fc4cce133f4539e44dde12b919a0019ba6e925",
          "message": "chat example (#46)\n\n* working toward chat\r\n\r\n* simple chat working\r\n\r\n* rebase working toward chat\r\n\r\n* msgs sent to server\r\n\r\n* simple chat working\r\n\r\n* finish merge\r\n\r\n* wip chat\r\n\r\n* chat v0.0\r\n\r\n* unused imports\r\n\r\n* clean up chat\r\n\r\n* make clippy happy",
          "timestamp": "2022-01-21T03:04:08Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/89fc4cce133f4539e44dde12b919a0019ba6e925"
        },
        "date": 1642909630721,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 432328,
            "range": "± 8013",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 208470804,
            "range": "± 1148294",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 197752254,
            "range": "± 5608160",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9951998,
            "range": "± 71745",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 49186306,
            "range": "± 664578",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55733281,
            "range": "± 900498",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16592954,
            "range": "± 236552",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2936259,
            "range": "± 22576",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3079790,
            "range": "± 19700",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "89fc4cce133f4539e44dde12b919a0019ba6e925",
          "message": "chat example (#46)\n\n* working toward chat\r\n\r\n* simple chat working\r\n\r\n* rebase working toward chat\r\n\r\n* msgs sent to server\r\n\r\n* simple chat working\r\n\r\n* finish merge\r\n\r\n* wip chat\r\n\r\n* chat v0.0\r\n\r\n* unused imports\r\n\r\n* clean up chat\r\n\r\n* make clippy happy",
          "timestamp": "2022-01-21T03:04:08Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/89fc4cce133f4539e44dde12b919a0019ba6e925"
        },
        "date": 1642996067134,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 437658,
            "range": "± 8866",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 212264335,
            "range": "± 4749302",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 198403804,
            "range": "± 2683044",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10989660,
            "range": "± 150188",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 50962377,
            "range": "± 1657256",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55356802,
            "range": "± 1310852",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17178007,
            "range": "± 304291",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2965886,
            "range": "± 48173",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3129603,
            "range": "± 40671",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643082428569,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 325837,
            "range": "± 20566",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204741577,
            "range": "± 7373422",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 207667620,
            "range": "± 16247411",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9706291,
            "range": "± 575613",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44193988,
            "range": "± 2245900",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57258301,
            "range": "± 4277657",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14562443,
            "range": "± 738338",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2712275,
            "range": "± 140473",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2834889,
            "range": "± 142472",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643168926210,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 367737,
            "range": "± 22008",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 234948854,
            "range": "± 5377406",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 231187590,
            "range": "± 5370260",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10345023,
            "range": "± 477658",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 54158962,
            "range": "± 3015319",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 63976965,
            "range": "± 3625442",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17089395,
            "range": "± 709421",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3072861,
            "range": "± 160661",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3247713,
            "range": "± 152229",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643255198113,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375596,
            "range": "± 4063",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 190858796,
            "range": "± 331960",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 176599249,
            "range": "± 8145920",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8275370,
            "range": "± 39680",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 48449569,
            "range": "± 660963",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48800789,
            "range": "± 426393",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691375,
            "range": "± 4460",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2624196,
            "range": "± 3367",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2684777,
            "range": "± 2766",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643341595333,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375354,
            "range": "± 3369",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 189035028,
            "range": "± 764831",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 176704093,
            "range": "± 1413836",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8946380,
            "range": "± 11218",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43658885,
            "range": "± 294771",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48202816,
            "range": "± 436853",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685399,
            "range": "± 8929",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2579063,
            "range": "± 2687",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2630552,
            "range": "± 2077",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643427990591,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 260999,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166783436,
            "range": "± 857270",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 156087152,
            "range": "± 24171542",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8086969,
            "range": "± 33446",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42104118,
            "range": "± 235588",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57343757,
            "range": "± 40961",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18206958,
            "range": "± 37196",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2850558,
            "range": "± 9417",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2950872,
            "range": "± 3379",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643514424543,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451116,
            "range": "± 2480",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 213235908,
            "range": "± 1230740",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 205647787,
            "range": "± 2579003",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9089751,
            "range": "± 44860",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 51848027,
            "range": "± 282124",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58782741,
            "range": "± 665818",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17612210,
            "range": "± 108384",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3083379,
            "range": "± 22513",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3163610,
            "range": "± 18232",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b09dec11aa4e7df535ad4d4fd4be958821b9a31d",
          "message": "First attempt at an exchange-like operator (#56)\n\nI'm not really sure how best to generalize this, which is something we will\r\nprobably have to figure out.",
          "timestamp": "2022-01-24T17:41:29Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b09dec11aa4e7df535ad4d4fd4be958821b9a31d"
        },
        "date": 1643600850959,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 450532,
            "range": "± 4268",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 207686738,
            "range": "± 2228640",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 204335230,
            "range": "± 8620142",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9229386,
            "range": "± 38549",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 52619956,
            "range": "± 218298",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59260209,
            "range": "± 556578",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17632286,
            "range": "± 17952",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3082147,
            "range": "± 37308",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3219378,
            "range": "± 4564",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6725322d6eb3405f93b2a4c5790b71c9d4e0a9b1",
          "message": "Ignore &mut Vec clippy warning",
          "timestamp": "2022-01-31T21:57:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6725322d6eb3405f93b2a4c5790b71c9d4e0a9b1"
        },
        "date": 1643687195317,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261116,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168672250,
            "range": "± 1249701",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 159208698,
            "range": "± 3133710",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8642540,
            "range": "± 30575",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42256037,
            "range": "± 165556",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57445976,
            "range": "± 123679",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18218004,
            "range": "± 32572",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2932560,
            "range": "± 5692",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3015745,
            "range": "± 4419",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6725322d6eb3405f93b2a4c5790b71c9d4e0a9b1",
          "message": "Ignore &mut Vec clippy warning",
          "timestamp": "2022-01-31T21:57:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6725322d6eb3405f93b2a4c5790b71c9d4e0a9b1"
        },
        "date": 1643773592382,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 294742,
            "range": "± 11540",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165000667,
            "range": "± 662504",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 155011070,
            "range": "± 5495053",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8074334,
            "range": "± 52623",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41988819,
            "range": "± 402800",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57632417,
            "range": "± 105905",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18215827,
            "range": "± 37287",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2869760,
            "range": "± 4011",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2924860,
            "range": "± 2745",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8c7bf017ad019f104d47fc46181231db3136a3e6",
          "message": "Add a `broadcast` operator (#61)\n\nNot much to this one, basically just a simplification of exchange.",
          "timestamp": "2022-02-02T21:26:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8c7bf017ad019f104d47fc46181231db3136a3e6"
        },
        "date": 1643860022151,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375684,
            "range": "± 2452",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195964510,
            "range": "± 752311",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 187928165,
            "range": "± 5810459",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7593906,
            "range": "± 25891",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43160100,
            "range": "± 357374",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47956726,
            "range": "± 462207",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679381,
            "range": "± 12815",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2584291,
            "range": "± 2768",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2618905,
            "range": "± 2301",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7bd4f89371546b380a41f44e8641a5512194d93a",
          "message": "Remove unused imports",
          "timestamp": "2022-02-04T00:25:09Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7bd4f89371546b380a41f44e8641a5512194d93a"
        },
        "date": 1643946404303,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 447826,
            "range": "± 7577",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 214841933,
            "range": "± 932827",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 127226894,
            "range": "± 511708",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10226852,
            "range": "± 207423",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42664938,
            "range": "± 130346",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 71409522,
            "range": "± 858548",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17637557,
            "range": "± 5320",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3208865,
            "range": "± 629918",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3200678,
            "range": "± 7422",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7bd4f89371546b380a41f44e8641a5512194d93a",
          "message": "Remove unused imports",
          "timestamp": "2022-02-04T00:25:09Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7bd4f89371546b380a41f44e8641a5512194d93a"
        },
        "date": 1643995768585,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 338163,
            "range": "± 12066",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186618739,
            "range": "± 5803584",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104974088,
            "range": "± 3388302",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9024478,
            "range": "± 336149",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36284403,
            "range": "± 1517558",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56893266,
            "range": "± 1821397",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16082565,
            "range": "± 778685",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2901926,
            "range": "± 168285",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2915351,
            "range": "± 111795",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1135b664fba1e5480ff6a493fd62f720c26b9ead",
          "message": "Update add_channel_input to take SendPort as arg instead of returning RecvPort",
          "timestamp": "2022-02-04T21:01:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1135b664fba1e5480ff6a493fd62f720c26b9ead"
        },
        "date": 1644032834410,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 379137,
            "range": "± 23707",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206582844,
            "range": "± 5141470",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121089765,
            "range": "± 3270767",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10899775,
            "range": "± 478136",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41246452,
            "range": "± 1021047",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76442493,
            "range": "± 2238323",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18314729,
            "range": "± 683556",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3302606,
            "range": "± 122395",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3327855,
            "range": "± 91681",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1135b664fba1e5480ff6a493fd62f720c26b9ead",
          "message": "Update add_channel_input to take SendPort as arg instead of returning RecvPort",
          "timestamp": "2022-02-04T21:01:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1135b664fba1e5480ff6a493fd62f720c26b9ead"
        },
        "date": 1644119207541,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 423185,
            "range": "± 9632",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 205927021,
            "range": "± 2988068",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 110896999,
            "range": "± 2258191",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8460907,
            "range": "± 465115",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41127843,
            "range": "± 1204923",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66082128,
            "range": "± 2963226",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15926333,
            "range": "± 475396",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2849168,
            "range": "± 85603",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2933907,
            "range": "± 99481",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1135b664fba1e5480ff6a493fd62f720c26b9ead",
          "message": "Update add_channel_input to take SendPort as arg instead of returning RecvPort",
          "timestamp": "2022-02-04T21:01:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1135b664fba1e5480ff6a493fd62f720c26b9ead"
        },
        "date": 1644205667182,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 369659,
            "range": "± 21238",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 228592753,
            "range": "± 7792085",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125635780,
            "range": "± 3812323",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9794951,
            "range": "± 645419",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40904272,
            "range": "± 2314440",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72335325,
            "range": "± 4690687",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16536760,
            "range": "± 987604",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3149051,
            "range": "± 174601",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3179820,
            "range": "± 168088",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a8dd6f485e5626688e285d28aeaa11b41e84759d",
          "message": "Add add_input_from_stream() to builder, fix #52",
          "timestamp": "2022-02-07T20:53:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a8dd6f485e5626688e285d28aeaa11b41e84759d"
        },
        "date": 1644292029946,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375918,
            "range": "± 2657",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 202580534,
            "range": "± 498302",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120082241,
            "range": "± 880473",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7951688,
            "range": "± 32368",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37655636,
            "range": "± 119262",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 74349186,
            "range": "± 1385749",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14698450,
            "range": "± 8641",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2730489,
            "range": "± 23750",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2752886,
            "range": "± 41120",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a61638aec9569a55f149feba3aeea55021ce35dc",
          "message": "Add context to surface build\n\nThis will be needed once more features are added to context.\nWe also have the option to remove the wonky handoff list structure via\nthis.",
          "timestamp": "2022-02-09T00:26:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a61638aec9569a55f149feba3aeea55021ce35dc"
        },
        "date": 1644378470334,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378696,
            "range": "± 49964",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 201907115,
            "range": "± 3684623",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113771954,
            "range": "± 2776845",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10412288,
            "range": "± 438249",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39435040,
            "range": "± 1056956",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72191844,
            "range": "± 2267171",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17883674,
            "range": "± 588797",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3147434,
            "range": "± 135908",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3287695,
            "range": "± 140894",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "92b4cfb043c6789497f74d429d7f3fa045d87355",
          "message": "Temporarily pin toolchain to nightly-2022-02-09",
          "timestamp": "2022-02-10T23:58:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/92b4cfb043c6789497f74d429d7f3fa045d87355"
        },
        "date": 1644637666373,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 354605,
            "range": "± 62007",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 219315918,
            "range": "± 13031114",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123131419,
            "range": "± 14228200",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10702579,
            "range": "± 1396649",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46352444,
            "range": "± 6108831",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 73439437,
            "range": "± 8528204",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16883680,
            "range": "± 2289487",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2997898,
            "range": "± 484044",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3373845,
            "range": "± 252902",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "92b4cfb043c6789497f74d429d7f3fa045d87355",
          "message": "Temporarily pin toolchain to nightly-2022-02-09",
          "timestamp": "2022-02-10T23:58:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/92b4cfb043c6789497f74d429d7f3fa045d87355"
        },
        "date": 1644724097624,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 385929,
            "range": "± 24664",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 202328896,
            "range": "± 4884044",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108650735,
            "range": "± 4030943",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11389079,
            "range": "± 414164",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 46988025,
            "range": "± 1838565",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 73134443,
            "range": "± 2758987",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17703107,
            "range": "± 598712",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3219251,
            "range": "± 123510",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3246591,
            "range": "± 182785",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "92b4cfb043c6789497f74d429d7f3fa045d87355",
          "message": "Temporarily pin toolchain to nightly-2022-02-09",
          "timestamp": "2022-02-10T23:58:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/92b4cfb043c6789497f74d429d7f3fa045d87355"
        },
        "date": 1644810419286,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375914,
            "range": "± 2528",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199841608,
            "range": "± 1641066",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 118659005,
            "range": "± 1025099",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9317505,
            "range": "± 195093",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43065481,
            "range": "± 121284",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61667739,
            "range": "± 1532728",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14693518,
            "range": "± 11070",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2650380,
            "range": "± 4182",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2694432,
            "range": "± 4445",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9941288a1c5ba86d322a13f6527e16de2a096de4",
          "message": "Initial pass of KVS (#77)\n\nThis is a very basic version of an Anna-like KVS. It doesn't actually use the\r\nHydroflow dataflow mechanisms yet because I am still trying to understand how\r\nthe system works fundamentally. I think the translation into dataflow should be\r\npretty straightforward.",
          "timestamp": "2022-02-14T20:31:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9941288a1c5ba86d322a13f6527e16de2a096de4"
        },
        "date": 1644896879660,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 354861,
            "range": "± 16978",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198059446,
            "range": "± 7579848",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105933768,
            "range": "± 3954585",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10069672,
            "range": "± 546475",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43872091,
            "range": "± 1552193",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72289892,
            "range": "± 4661225",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17778502,
            "range": "± 709609",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2998125,
            "range": "± 100900",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3000684,
            "range": "± 158948",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "efa9a86a519485a92e07c45be932beea752340d8",
          "message": "Remove obsolete \"variadic_generics\" feature",
          "timestamp": "2022-02-15T22:07:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/efa9a86a519485a92e07c45be932beea752340d8"
        },
        "date": 1644983300307,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 458621,
            "range": "± 8896",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216599051,
            "range": "± 1033710",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 129535550,
            "range": "± 2686350",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11096190,
            "range": "± 222518",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 50747000,
            "range": "± 250435",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 78588598,
            "range": "± 1175959",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17916416,
            "range": "± 627098",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3249506,
            "range": "± 87377",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3173693,
            "range": "± 17601",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "David Chu",
            "username": "davidchuyaya",
            "email": "davidchuyaya@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2b9fac8368495cf7b4e3c9a1c5e5ef7a6097dc0e",
          "message": "Create README.md",
          "timestamp": "2022-02-16T22:33:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/2b9fac8368495cf7b4e3c9a1c5e5ef7a6097dc0e"
        },
        "date": 1645069638308,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376158,
            "range": "± 2413",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206945721,
            "range": "± 907919",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 124175124,
            "range": "± 912280",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9138499,
            "range": "± 437896",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44696450,
            "range": "± 293514",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 75163876,
            "range": "± 943667",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14709830,
            "range": "± 11346",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2750744,
            "range": "± 37589",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2834208,
            "range": "± 53612",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645156000457,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261478,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 176731966,
            "range": "± 777911",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105287243,
            "range": "± 709966",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8238134,
            "range": "± 118872",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44536255,
            "range": "± 213463",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 78922471,
            "range": "± 388732",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224848,
            "range": "± 12839",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2855519,
            "range": "± 6115",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2926402,
            "range": "± 2766",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645242455226,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 381077,
            "range": "± 20517",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204093078,
            "range": "± 7983609",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109638587,
            "range": "± 4103584",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10715086,
            "range": "± 550592",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 47704931,
            "range": "± 1557095",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 77191949,
            "range": "± 3066152",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18057740,
            "range": "± 719146",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3212945,
            "range": "± 189511",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3273897,
            "range": "± 161446",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645328818857,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 333858,
            "range": "± 21344",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 211570244,
            "range": "± 8819544",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119633089,
            "range": "± 6397952",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10185981,
            "range": "± 579194",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41278485,
            "range": "± 2281161",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 65061615,
            "range": "± 4322367",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15019646,
            "range": "± 730276",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2846873,
            "range": "± 132664",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2879165,
            "range": "± 179857",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645415244840,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 356870,
            "range": "± 26156",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200809864,
            "range": "± 7831254",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109467109,
            "range": "± 3965114",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9578977,
            "range": "± 496827",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43352260,
            "range": "± 1806951",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58707286,
            "range": "± 2273029",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16192301,
            "range": "± 467340",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2930508,
            "range": "± 123859",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2940124,
            "range": "± 61261",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645501556635,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331387,
            "range": "± 2670",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 175753758,
            "range": "± 598961",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106292275,
            "range": "± 4750690",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8433670,
            "range": "± 30249",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38145894,
            "range": "± 101980",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55826728,
            "range": "± 181770",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12958805,
            "range": "± 8880",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2345658,
            "range": "± 2186",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2372301,
            "range": "± 2418",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c409c3f5bde07685d87899a1d03244426c09c322",
          "message": "Use `Name: Into<Cow<'static, str>>` generic to avoid `.into()`s",
          "timestamp": "2022-02-16T23:59:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c409c3f5bde07685d87899a1d03244426c09c322"
        },
        "date": 1645587966443,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375805,
            "range": "± 2834",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186303891,
            "range": "± 1104804",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108227413,
            "range": "± 1715938",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8443681,
            "range": "± 36668",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38140543,
            "range": "± 410176",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56626084,
            "range": "± 881601",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12972895,
            "range": "± 27147",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2357663,
            "range": "± 6538",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2380828,
            "range": "± 15715",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Conor Power",
            "username": "conor-23",
            "email": "94084298+conor-23@users.noreply.github.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c1884eebb1ed63e6202e2d1a422905c6e1529e44",
          "message": "add three_clique hydroflow example (#84)\n\n* add three_clique hydroflow example\r\n* fix formatting of three_clique example",
          "timestamp": "2022-02-23T19:22:00Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c1884eebb1ed63e6202e2d1a422905c6e1529e44"
        },
        "date": 1645674398842,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 329070,
            "range": "± 16551",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 211325277,
            "range": "± 12632434",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111420725,
            "range": "± 4515747",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8834384,
            "range": "± 399771",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38294697,
            "range": "± 1688611",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57309261,
            "range": "± 4093640",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 13752153,
            "range": "± 526158",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2616939,
            "range": "± 151064",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2651313,
            "range": "± 114268",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "54bafa4c2ce8c861ca9b176ac53d329acc3185c6",
          "message": "Add Two-Phase Commit example (#86)\n\n* two phase commit example\r\n\r\n* respond to PR comments\r\n\r\n* remove dead use line\r\n\r\n* fix comment formatting for linter\r\n\r\n* remove redundant message construction",
          "timestamp": "2022-02-24T08:07:06Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/54bafa4c2ce8c861ca9b176ac53d329acc3185c6"
        },
        "date": 1645760785081,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374964,
            "range": "± 2820",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198396331,
            "range": "± 521534",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119584629,
            "range": "± 1182094",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8229389,
            "range": "± 243714",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41707981,
            "range": "± 51946",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60087720,
            "range": "± 751849",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687719,
            "range": "± 7195",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2716388,
            "range": "± 3400",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2661821,
            "range": "± 3800",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "54bafa4c2ce8c861ca9b176ac53d329acc3185c6",
          "message": "Add Two-Phase Commit example (#86)\n\n* two phase commit example\r\n\r\n* respond to PR comments\r\n\r\n* remove dead use line\r\n\r\n* fix comment formatting for linter\r\n\r\n* remove redundant message construction",
          "timestamp": "2022-02-24T08:07:06Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/54bafa4c2ce8c861ca9b176ac53d329acc3185c6"
        },
        "date": 1645847231559,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 446612,
            "range": "± 6399",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 213415556,
            "range": "± 794956",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 124693412,
            "range": "± 1299852",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11448931,
            "range": "± 322347",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 48974480,
            "range": "± 102959",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 71832423,
            "range": "± 695751",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17637872,
            "range": "± 29820",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3184254,
            "range": "± 10395",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3193478,
            "range": "± 7978",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "54bafa4c2ce8c861ca9b176ac53d329acc3185c6",
          "message": "Add Two-Phase Commit example (#86)\n\n* two phase commit example\r\n\r\n* respond to PR comments\r\n\r\n* remove dead use line\r\n\r\n* fix comment formatting for linter\r\n\r\n* remove redundant message construction",
          "timestamp": "2022-02-24T08:07:06Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/54bafa4c2ce8c861ca9b176ac53d329acc3185c6"
        },
        "date": 1645933592097,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 286451,
            "range": "± 13257",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 171757290,
            "range": "± 3561921",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 95929807,
            "range": "± 831010",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8464508,
            "range": "± 86593",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44626292,
            "range": "± 47042",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 78621292,
            "range": "± 659487",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18227451,
            "range": "± 13022",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2925324,
            "range": "± 6470",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2945552,
            "range": "± 6013",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "54bafa4c2ce8c861ca9b176ac53d329acc3185c6",
          "message": "Add Two-Phase Commit example (#86)\n\n* two phase commit example\r\n\r\n* respond to PR comments\r\n\r\n* remove dead use line\r\n\r\n* fix comment formatting for linter\r\n\r\n* remove redundant message construction",
          "timestamp": "2022-02-24T08:07:06Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/54bafa4c2ce8c861ca9b176ac53d329acc3185c6"
        },
        "date": 1646019994013,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375742,
            "range": "± 2684",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188816832,
            "range": "± 736064",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111047538,
            "range": "± 984224",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7652961,
            "range": "± 43509",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42697437,
            "range": "± 348820",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59919372,
            "range": "± 448785",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688201,
            "range": "± 43172",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2684538,
            "range": "± 24465",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2630942,
            "range": "± 8575",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f8efbc50837723bb0b4b721a897bcf2c6e1065c8",
          "message": "Fix hf.run_async() busy spinning.\n\nThis replaces the std mpsc event channels with tokio unbounded channels\nso we can .await them.\n\nFixes #87",
          "timestamp": "2022-02-28T21:15:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f8efbc50837723bb0b4b721a897bcf2c6e1065c8"
        },
        "date": 1646106489203,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371801,
            "range": "± 21620",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198121650,
            "range": "± 8253665",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113533322,
            "range": "± 5321767",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10900233,
            "range": "± 676280",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38545375,
            "range": "± 2511957",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57279680,
            "range": "± 3662607",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17300729,
            "range": "± 840868",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3256632,
            "range": "± 233309",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3213511,
            "range": "± 120736",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f8efbc50837723bb0b4b721a897bcf2c6e1065c8",
          "message": "Fix hf.run_async() busy spinning.\n\nThis replaces the std mpsc event channels with tokio unbounded channels\nso we can .await them.\n\nFixes #87",
          "timestamp": "2022-02-28T21:15:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f8efbc50837723bb0b4b721a897bcf2c6e1065c8"
        },
        "date": 1646192765211,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261535,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 170415754,
            "range": "± 1773523",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101881202,
            "range": "± 547003",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10838169,
            "range": "± 116055",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34472630,
            "range": "± 118710",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58602724,
            "range": "± 501974",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224779,
            "range": "± 49389",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2931234,
            "range": "± 5922",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2976135,
            "range": "± 7592",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f8efbc50837723bb0b4b721a897bcf2c6e1065c8",
          "message": "Fix hf.run_async() busy spinning.\n\nThis replaces the std mpsc event channels with tokio unbounded channels\nso we can .await them.\n\nFixes #87",
          "timestamp": "2022-02-28T21:15:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f8efbc50837723bb0b4b721a897bcf2c6e1065c8"
        },
        "date": 1646279233718,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 437043,
            "range": "± 8053",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 212612163,
            "range": "± 1732496",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126262894,
            "range": "± 5011646",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12894175,
            "range": "± 717859",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41704333,
            "range": "± 439043",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56022300,
            "range": "± 997282",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17017291,
            "range": "± 335582",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3140561,
            "range": "± 53309",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3095240,
            "range": "± 45969",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a4d94a4bad2483eb11bbe86bc9e7c05f93a30bc8",
          "message": "Rewrite KVS in dataflow (#91)\n\n* add stream join operator\r\n\r\n* Add KVS in dataflow\r\n\r\nThis is a first pass at a dataflow based KVS. It is missing a bunch of clock\r\nstuff, but I was getting anxious about not merging this so here's an early\r\nversion of it.\r\n\r\nThe structure of the program is roughly as follows:\r\n\r\n```mermaid\r\ngraph BT\r\n    Y[Read Requests] --> Z[Streaming Join]\r\n    X[Received Batches] --> Z\r\n    Z --> Q[Respond to Read Request]\r\n\r\n    F[Write Requests] --> B\r\n    A[Epoch Timer] --> B[Batcher / Merger]\r\n    B --> C[Join]\r\n    G[Ownership] --> C\r\n    C --> E[Network Shuffle]\r\n```",
          "timestamp": "2022-03-03T20:06:44Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a4d94a4bad2483eb11bbe86bc9e7c05f93a30bc8"
        },
        "date": 1646365678686,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 379450,
            "range": "± 14058",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 210470266,
            "range": "± 6160975",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120967636,
            "range": "± 3546230",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13315614,
            "range": "± 517405",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39535762,
            "range": "± 1265805",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61835552,
            "range": "± 3062415",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18045364,
            "range": "± 426834",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3152387,
            "range": "± 88568",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3196559,
            "range": "± 127362",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d39d44d76355db3e0b9b5c1317dbdd9ba314857c",
          "message": "Rename scheduler `ready_queue` to `stratum_queues`",
          "timestamp": "2022-03-04T23:32:17Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d39d44d76355db3e0b9b5c1317dbdd9ba314857c"
        },
        "date": 1646451963256,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375616,
            "range": "± 3170",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199429704,
            "range": "± 411496",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106319427,
            "range": "± 526125",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10025187,
            "range": "± 180849",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36297601,
            "range": "± 46305",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47959006,
            "range": "± 211671",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14678371,
            "range": "± 12411",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2655993,
            "range": "± 2337",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2690682,
            "range": "± 1967",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d39d44d76355db3e0b9b5c1317dbdd9ba314857c",
          "message": "Rename scheduler `ready_queue` to `stratum_queues`",
          "timestamp": "2022-03-04T23:32:17Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d39d44d76355db3e0b9b5c1317dbdd9ba314857c"
        },
        "date": 1646538446532,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 359772,
            "range": "± 35852",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 192434094,
            "range": "± 7588234",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104520419,
            "range": "± 4531994",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13270178,
            "range": "± 720454",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36255589,
            "range": "± 1727670",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54700123,
            "range": "± 3140828",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16403421,
            "range": "± 768691",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3008738,
            "range": "± 141914",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3174665,
            "range": "± 163335",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d39d44d76355db3e0b9b5c1317dbdd9ba314857c",
          "message": "Rename scheduler `ready_queue` to `stratum_queues`",
          "timestamp": "2022-03-04T23:32:17Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d39d44d76355db3e0b9b5c1317dbdd9ba314857c"
        },
        "date": 1646624778047,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 317983,
            "range": "± 5646",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166730893,
            "range": "± 744373",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98867404,
            "range": "± 778109",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10302585,
            "range": "± 165247",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34000483,
            "range": "± 57442",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57664850,
            "range": "± 50502",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18211569,
            "range": "± 28287",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2910838,
            "range": "± 5547",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2933150,
            "range": "± 5431",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5dec91286550c84ae11d569c82c09bbb6efcb23e",
          "message": "CI publish design_docs on docs site",
          "timestamp": "2022-03-07T19:30:30Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5dec91286550c84ae11d569c82c09bbb6efcb23e"
        },
        "date": 1646711180104,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 262941,
            "range": "± 8577",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 171196585,
            "range": "± 1084067",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 102110779,
            "range": "± 2991927",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10751412,
            "range": "± 147916",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34387643,
            "range": "± 538072",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57990038,
            "range": "± 371902",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18228114,
            "range": "± 14457",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2923212,
            "range": "± 16920",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2928456,
            "range": "± 4345",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "71fa71a73cdeade8b18bfa4e5d051f5f0976c990",
          "message": "Make groupby tests more relational-looking (#100)\n\n* Make groupby tests more relational-looking\r\n* Make even more relational with itertools\r\n* Change groupby to use tuples instead of key fn",
          "timestamp": "2022-03-08T23:22:21Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/71fa71a73cdeade8b18bfa4e5d051f5f0976c990"
        },
        "date": 1646797594470,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 400503,
            "range": "± 13008",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 204731429,
            "range": "± 3500706",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116526577,
            "range": "± 2078274",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11530188,
            "range": "± 314517",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39279123,
            "range": "± 1028637",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50929475,
            "range": "± 1482305",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15647178,
            "range": "± 418191",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2864639,
            "range": "± 86671",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2798608,
            "range": "± 79628",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "54c70707ffb8e9c00d71f8ed7c389f94e04846c2",
          "message": "Move KVS to examples (#101)\n\nAlso fixes some lints",
          "timestamp": "2022-03-09T21:44:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/54c70707ffb8e9c00d71f8ed7c389f94e04846c2"
        },
        "date": 1646884028400,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 260930,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 158544551,
            "range": "± 1260168",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97706078,
            "range": "± 598919",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11020257,
            "range": "± 254178",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34264839,
            "range": "± 193027",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56354375,
            "range": "± 124893",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209865,
            "range": "± 23119",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2930900,
            "range": "± 9269",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2956538,
            "range": "± 2843",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a65a3c13c0e0bd37748209ad7d40b5864b4edcd3",
          "message": "Clean up old code in chat client.rs",
          "timestamp": "2022-03-10T22:41:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a65a3c13c0e0bd37748209ad7d40b5864b4edcd3"
        },
        "date": 1646970432049,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 367377,
            "range": "± 17838",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 202023244,
            "range": "± 4137630",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113906172,
            "range": "± 2981049",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13083675,
            "range": "± 572924",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38259429,
            "range": "± 1189118",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56274793,
            "range": "± 3517944",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17954191,
            "range": "± 1112190",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3262313,
            "range": "± 238993",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3253530,
            "range": "± 111249",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f44c50a5dd9d4fe44b0095b8d06b75818e77debb",
          "message": "Update where clause position\n\nhttps://github.com/rust-lang/rust/issues/89122",
          "timestamp": "2022-03-07T21:44:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f44c50a5dd9d4fe44b0095b8d06b75818e77debb"
        },
        "date": 1647056807416,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 336641,
            "range": "± 12414",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 191240370,
            "range": "± 6171804",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111283124,
            "range": "± 2509414",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12255626,
            "range": "± 784380",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35236346,
            "range": "± 1076948",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52100257,
            "range": "± 1877374",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16027665,
            "range": "± 452816",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2963238,
            "range": "± 131106",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2970019,
            "range": "± 176715",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f44c50a5dd9d4fe44b0095b8d06b75818e77debb",
          "message": "Update where clause position\n\nhttps://github.com/rust-lang/rust/issues/89122",
          "timestamp": "2022-03-07T21:44:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f44c50a5dd9d4fe44b0095b8d06b75818e77debb"
        },
        "date": 1647143185518,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375837,
            "range": "± 2752",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 201538942,
            "range": "± 935935",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120923688,
            "range": "± 893060",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12361261,
            "range": "± 118644",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37206350,
            "range": "± 157260",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52571090,
            "range": "± 1446322",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14697720,
            "range": "± 10562",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2671435,
            "range": "± 5163",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2664627,
            "range": "± 4094",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f44c50a5dd9d4fe44b0095b8d06b75818e77debb",
          "message": "Update where clause position\n\nhttps://github.com/rust-lang/rust/issues/89122",
          "timestamp": "2022-03-07T21:44:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f44c50a5dd9d4fe44b0095b8d06b75818e77debb"
        },
        "date": 1647229672402,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 443889,
            "range": "± 6121",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 215171323,
            "range": "± 1331855",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123380861,
            "range": "± 843328",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14311749,
            "range": "± 380133",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41791723,
            "range": "± 475177",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56502796,
            "range": "± 800181",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17320745,
            "range": "± 213917",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3130064,
            "range": "± 47192",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3145050,
            "range": "± 38805",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "76168d37f49c9767625de33e1f87ea99f7f5d7a2",
          "message": "Add dev setup to README.md",
          "timestamp": "2022-03-11T21:56:54Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/76168d37f49c9767625de33e1f87ea99f7f5d7a2"
        },
        "date": 1647315964804,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331565,
            "range": "± 2982",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186668045,
            "range": "± 1090381",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97958110,
            "range": "± 1092191",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10802782,
            "range": "± 140579",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33379100,
            "range": "± 144248",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 44006553,
            "range": "± 735844",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12960506,
            "range": "± 67801",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2481989,
            "range": "± 17015",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2420074,
            "range": "± 2927",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9857a394f4814388997353328080e9885f46f0d6",
          "message": "dataflow diagrams for 2pc readme",
          "timestamp": "2022-03-15T00:16:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9857a394f4814388997353328080e9885f46f0d6"
        },
        "date": 1647402559222,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376459,
            "range": "± 16229",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 211479662,
            "range": "± 5305297",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121978789,
            "range": "± 4104164",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13724528,
            "range": "± 754793",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40392939,
            "range": "± 1929029",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 63012171,
            "range": "± 3971991",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18273801,
            "range": "± 641879",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3380606,
            "range": "± 121058",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3392017,
            "range": "± 171762",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "34251436edfe512bb017cfa9849e65a85b7bbbfd",
          "message": "Combine old 'hof with context under new 'ctx lifetime",
          "timestamp": "2022-03-16T21:27:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/34251436edfe512bb017cfa9849e65a85b7bbbfd"
        },
        "date": 1647488791567,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 264783,
            "range": "± 3527",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 171750692,
            "range": "± 2082575",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101617215,
            "range": "± 1321709",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10486940,
            "range": "± 134909",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34444041,
            "range": "± 350885",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58141202,
            "range": "± 534459",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18405497,
            "range": "± 154431",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2910914,
            "range": "± 34293",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2943305,
            "range": "± 49638",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eebd4075e83808d51b40b4850e60ddba52f5862c",
          "message": "kvs: pull benchmark into separate file (#116)\n\nAlso remove the reporting of latency information for now, since it was\r\nvulnerable to coordinated omission and is not reliable.",
          "timestamp": "2022-03-17T20:47:14Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/eebd4075e83808d51b40b4850e60ddba52f5862c"
        },
        "date": 1647575223435,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 263853,
            "range": "± 4297",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 162168119,
            "range": "± 607359",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101192919,
            "range": "± 3365780",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10393411,
            "range": "± 136048",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34105649,
            "range": "± 115234",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61240947,
            "range": "± 240375",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18228705,
            "range": "± 11418",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2911718,
            "range": "± 6315",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2919391,
            "range": "± 3651",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5618b3723674e3fc14f2378fe27319c06b66d32a",
          "message": "Add partition_with_context, change to use FnMut instead of Fn",
          "timestamp": "2022-03-17T22:23:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5618b3723674e3fc14f2378fe27319c06b66d32a"
        },
        "date": 1647661678326,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 413443,
            "range": "± 24519",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 253788332,
            "range": "± 9449767",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 150400636,
            "range": "± 7539155",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14495138,
            "range": "± 935228",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 44329178,
            "range": "± 2718475",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72741690,
            "range": "± 5691752",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 19232391,
            "range": "± 941644",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3682836,
            "range": "± 247498",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3698188,
            "range": "± 294906",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5618b3723674e3fc14f2378fe27319c06b66d32a",
          "message": "Add partition_with_context, change to use FnMut instead of Fn",
          "timestamp": "2022-03-17T22:23:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5618b3723674e3fc14f2378fe27319c06b66d32a"
        },
        "date": 1647747969065,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375111,
            "range": "± 2698",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198733020,
            "range": "± 525694",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 118427309,
            "range": "± 2036049",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11505485,
            "range": "± 35117",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36005710,
            "range": "± 63363",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47398057,
            "range": "± 473133",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14690855,
            "range": "± 4583",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2800691,
            "range": "± 5527",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3431501,
            "range": "± 2514",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5618b3723674e3fc14f2378fe27319c06b66d32a",
          "message": "Add partition_with_context, change to use FnMut instead of Fn",
          "timestamp": "2022-03-17T22:23:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5618b3723674e3fc14f2378fe27319c06b66d32a"
        },
        "date": 1647834575719,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378524,
            "range": "± 16375",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 206637920,
            "range": "± 6136370",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117076774,
            "range": "± 3883527",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13477534,
            "range": "± 460132",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40456304,
            "range": "± 1366491",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48908554,
            "range": "± 2939145",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17876775,
            "range": "± 545341",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3312626,
            "range": "± 129434",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3431697,
            "range": "± 167316",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "53ba0257c4413174d3edf8a3a3da9afc60d30bfc",
          "message": "fixup! Include gh-pages files thru build",
          "timestamp": "2022-03-21T23:25:06Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/53ba0257c4413174d3edf8a3a3da9afc60d30bfc"
        },
        "date": 1647920939995,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 286076,
            "range": "± 4614",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195835690,
            "range": "± 7309594",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104580481,
            "range": "± 2464816",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10046787,
            "range": "± 353110",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31090395,
            "range": "± 570438",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47406213,
            "range": "± 1644142",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 13388769,
            "range": "± 419977",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2671111,
            "range": "± 68438",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2672197,
            "range": "± 74228",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dcdba24da73c1e0d000d846e32ef1e853a1b2b6c",
          "message": "Add fold_epoch to surface API (#121)",
          "timestamp": "2022-03-22T21:49:03Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dcdba24da73c1e0d000d846e32ef1e853a1b2b6c"
        },
        "date": 1648007448379,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 264226,
            "range": "± 3931",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 172978961,
            "range": "± 4493487",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97804417,
            "range": "± 1446673",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10538298,
            "range": "± 177252",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34718101,
            "range": "± 162334",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58006450,
            "range": "± 835442",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18399067,
            "range": "± 321142",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2927681,
            "range": "± 35724",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2951178,
            "range": "± 32198",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "35fa5e9870006d46203771381ade4ba7615ec8f2",
          "message": "fixup! Use newtypes for Subgraph/Handoff/State IDs",
          "timestamp": "2022-03-23T21:09:25Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/35fa5e9870006d46203771381ade4ba7615ec8f2"
        },
        "date": 1648093687722,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374769,
            "range": "± 2641",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 239194147,
            "range": "± 1247653",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117338205,
            "range": "± 1175195",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10521543,
            "range": "± 36564",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36744627,
            "range": "± 218189",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61577402,
            "range": "± 242417",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682306,
            "range": "± 9583",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2651572,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2962908,
            "range": "± 19365",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "88fd58855209c2e4258d061d65cabe90c59bb605",
          "message": "Write introduction and setup portion of book\n\nMoved from README.md",
          "timestamp": "2022-03-21T23:16:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/88fd58855209c2e4258d061d65cabe90c59bb605"
        },
        "date": 1648152810873,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 419435,
            "range": "± 29477",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 279282428,
            "range": "± 7597963",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 132620009,
            "range": "± 3966722",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13695498,
            "range": "± 654653",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40836933,
            "range": "± 1349432",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 81432481,
            "range": "± 3109845",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18470335,
            "range": "± 1073044",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3443914,
            "range": "± 119680",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3517443,
            "range": "± 110083",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1cf48c4d89c5724a46793ec1faea81d3e88c5ec0",
          "message": "Use `NodeId` for mermaid graph instead of `usize`",
          "timestamp": "2022-03-23T21:08:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1cf48c4d89c5724a46793ec1faea81d3e88c5ec0"
        },
        "date": 1648180059446,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261334,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 222300475,
            "range": "± 1007610",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100437237,
            "range": "± 1012936",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10769684,
            "range": "± 88236",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34486394,
            "range": "± 117535",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 81534168,
            "range": "± 2246209",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224858,
            "range": "± 12048",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2864186,
            "range": "± 3022",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2924425,
            "range": "± 2999",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1cf48c4d89c5724a46793ec1faea81d3e88c5ec0",
          "message": "Use `NodeId` for mermaid graph instead of `usize`",
          "timestamp": "2022-03-23T21:08:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1cf48c4d89c5724a46793ec1faea81d3e88c5ec0"
        },
        "date": 1648266453819,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 316543,
            "range": "± 6546",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 222062729,
            "range": "± 1427067",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100960574,
            "range": "± 682889",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10837108,
            "range": "± 98676",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33947633,
            "range": "± 94828",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 83046964,
            "range": "± 908279",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18225731,
            "range": "± 12053",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2887227,
            "range": "± 18978",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2919284,
            "range": "± 2818",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1cf48c4d89c5724a46793ec1faea81d3e88c5ec0",
          "message": "Use `NodeId` for mermaid graph instead of `usize`",
          "timestamp": "2022-03-23T21:08:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1cf48c4d89c5724a46793ec1faea81d3e88c5ec0"
        },
        "date": 1648352852298,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 318482,
            "range": "± 5599",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 224188087,
            "range": "± 3608927",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101637772,
            "range": "± 1253483",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10870610,
            "range": "± 79128",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33995637,
            "range": "± 207565",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 83371715,
            "range": "± 595282",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224095,
            "range": "± 11556",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2866740,
            "range": "± 4628",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2897584,
            "range": "± 3863",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1cf48c4d89c5724a46793ec1faea81d3e88c5ec0",
          "message": "Use `NodeId` for mermaid graph instead of `usize`",
          "timestamp": "2022-03-23T21:08:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1cf48c4d89c5724a46793ec1faea81d3e88c5ec0"
        },
        "date": 1648439567903,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 432874,
            "range": "± 8559",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 239827214,
            "range": "± 3413281",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121824318,
            "range": "± 3028465",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11682096,
            "range": "± 442569",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40257542,
            "range": "± 1282386",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66567265,
            "range": "± 2692806",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15809874,
            "range": "± 606825",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2929942,
            "range": "± 123364",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2941955,
            "range": "± 118459",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "47536c487020b32366628dbc60c6d9b9178dd3c6",
          "message": "dot graphs and indents for mermaid (#127)\n\n* dot graphs\r\n\r\n* fix dot flags in READMEs\r\n\r\n* remove boilerplate comment\r\n\r\n* clean up graphing support\r\n\r\n* remove stray comment\r\n\r\n* address comments\r\n\r\n* address linter's alphabetization obsession",
          "timestamp": "2022-03-28T20:02:04Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/47536c487020b32366628dbc60c6d9b9178dd3c6"
        },
        "date": 1648525989249,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 364473,
            "range": "± 23538",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 238051775,
            "range": "± 7132277",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121205088,
            "range": "± 4265374",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13857754,
            "range": "± 631715",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40160613,
            "range": "± 1920726",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 74492673,
            "range": "± 3537462",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17803472,
            "range": "± 913669",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3250932,
            "range": "± 161413",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3221404,
            "range": "± 197845",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c33f81a10963ff2878b687fa9fd4e8bfc3a34fcd",
          "message": "fixup! Add another implementation of KVS (#122)",
          "timestamp": "2022-03-29T20:04:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c33f81a10963ff2878b687fa9fd4e8bfc3a34fcd"
        },
        "date": 1648612329528,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 428826,
            "range": "± 12281",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 250485583,
            "range": "± 4047299",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 122800641,
            "range": "± 2212304",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12021914,
            "range": "± 390078",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40608859,
            "range": "± 1162540",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76968237,
            "range": "± 2334913",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17206088,
            "range": "± 333190",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3007262,
            "range": "± 93234",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3057157,
            "range": "± 97361",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "bf37b4cd247c51e7cd1687db4fc8270b41a02228",
          "message": "Add book mermaid support, finish example 1",
          "timestamp": "2022-03-29T19:18:30Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/bf37b4cd247c51e7cd1687db4fc8270b41a02228"
        },
        "date": 1648698702400,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 469636,
            "range": "± 2770",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 249779736,
            "range": "± 1693337",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 127550592,
            "range": "± 2422484",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15869589,
            "range": "± 103969",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43411990,
            "range": "± 97742",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76236075,
            "range": "± 243974",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18341265,
            "range": "± 17548",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3303109,
            "range": "± 9452",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3326847,
            "range": "± 1885",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0a9b96cd76f378233e9ff8cb1a8e97ec6b2dab5e",
          "message": "Move diagram code into flow_graph",
          "timestamp": "2022-03-30T22:32:34Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0a9b96cd76f378233e9ff8cb1a8e97ec6b2dab5e"
        },
        "date": 1648785423020,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 381952,
            "range": "± 9170",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 246013742,
            "range": "± 4751619",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119123342,
            "range": "± 2278429",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13516213,
            "range": "± 251734",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40473095,
            "range": "± 1332431",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66403044,
            "range": "± 1275103",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209188,
            "range": "± 584558",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3375858,
            "range": "± 79235",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3384590,
            "range": "± 139112",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "3aa8744f64df8f8b8d39fbc0f3b9528770712b1c",
          "message": "Add vechandoff to builder::prelude",
          "timestamp": "2022-04-01T22:04:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3aa8744f64df8f8b8d39fbc0f3b9528770712b1c"
        },
        "date": 1648871426724,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331767,
            "range": "± 2779",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 232184703,
            "range": "± 793680",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116713457,
            "range": "± 931301",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11586707,
            "range": "± 38932",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37489580,
            "range": "± 254247",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66265434,
            "range": "± 769443",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14700259,
            "range": "± 6787",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2684856,
            "range": "± 11410",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2727553,
            "range": "± 19091",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "3aa8744f64df8f8b8d39fbc0f3b9528770712b1c",
          "message": "Add vechandoff to builder::prelude",
          "timestamp": "2022-04-01T22:04:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3aa8744f64df8f8b8d39fbc0f3b9528770712b1c"
        },
        "date": 1648957867997,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 466535,
            "range": "± 32759",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 260756555,
            "range": "± 1326242",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126893487,
            "range": "± 1501118",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13932995,
            "range": "± 42296",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43071933,
            "range": "± 130956",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 83143099,
            "range": "± 765250",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17641219,
            "range": "± 26546",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3484744,
            "range": "± 188875",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3432851,
            "range": "± 182235",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "3aa8744f64df8f8b8d39fbc0f3b9528770712b1c",
          "message": "Add vechandoff to builder::prelude",
          "timestamp": "2022-04-01T22:04:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3aa8744f64df8f8b8d39fbc0f3b9528770712b1c"
        },
        "date": 1649044392989,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 444786,
            "range": "± 5246",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 257214867,
            "range": "± 1529299",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126410416,
            "range": "± 1837921",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14473326,
            "range": "± 298539",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43351680,
            "range": "± 562779",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 80130682,
            "range": "± 1390605",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17483184,
            "range": "± 173990",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3143716,
            "range": "± 62011",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3131684,
            "range": "± 68228",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "d986d010aa0759ee9acf42368c21cd3480ec0618",
          "message": "Remove lifetime from Context, avoid need to construct instances",
          "timestamp": "2022-03-30T21:23:22Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d986d010aa0759ee9acf42368c21cd3480ec0618"
        },
        "date": 1649130625485,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375167,
            "range": "± 21499",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 209818590,
            "range": "± 2172032",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 112461706,
            "range": "± 1185912",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8927677,
            "range": "± 39314",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32099511,
            "range": "± 191457",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61786327,
            "range": "± 4028972",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14693403,
            "range": "± 12878",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2355286,
            "range": "± 5100",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2356979,
            "range": "± 2224",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f1b6e8dfec42d57c1a5396defb695c6a4a68e527",
          "message": "Include mdbook-mermaid in build",
          "timestamp": "2022-04-05T21:23:56Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f1b6e8dfec42d57c1a5396defb695c6a4a68e527"
        },
        "date": 1649217102013,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 312502,
            "range": "± 15504",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 220629733,
            "range": "± 1453406",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98466613,
            "range": "± 1301750",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10275549,
            "range": "± 104815",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34193326,
            "range": "± 196032",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 81963215,
            "range": "± 272217",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18219125,
            "range": "± 19119",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2861827,
            "range": "± 10899",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3027335,
            "range": "± 20029",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d020b01869532136cc7a0bf45629c86dc5c411c2",
          "message": "Re-add the \"raw\" implementation of the KVS (#137)\n\nThis one uses no Hydroflow at all. Similar to the original one, just factors\r\nsome stuff out.",
          "timestamp": "2022-04-06T22:28:15Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d020b01869532136cc7a0bf45629c86dc5c411c2"
        },
        "date": 1649303455413,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374929,
            "range": "± 2664",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199874139,
            "range": "± 494033",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114760491,
            "range": "± 1595219",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12657312,
            "range": "± 23969",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36410141,
            "range": "± 63615",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 65710618,
            "range": "± 624639",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687428,
            "range": "± 13358",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2654093,
            "range": "± 6944",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2693638,
            "range": "± 3084",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d020b01869532136cc7a0bf45629c86dc5c411c2",
          "message": "Re-add the \"raw\" implementation of the KVS (#137)\n\nThis one uses no Hydroflow at all. Similar to the original one, just factors\r\nsome stuff out.",
          "timestamp": "2022-04-06T22:28:15Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/d020b01869532136cc7a0bf45629c86dc5c411c2"
        },
        "date": 1649389955224,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370702,
            "range": "± 2969",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 197148131,
            "range": "± 1044885",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114707780,
            "range": "± 1030017",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10048813,
            "range": "± 50524",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36375476,
            "range": "± 156062",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61009180,
            "range": "± 402525",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685063,
            "range": "± 11227",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2672996,
            "range": "± 5048",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2678815,
            "range": "± 2946",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "ec81dca8daa9f3e790ca817f98325347ce6c8681",
          "message": "Finish book example_4",
          "timestamp": "2022-04-07T21:51:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/ec81dca8daa9f3e790ca817f98325347ce6c8681"
        },
        "date": 1649821949169,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375602,
            "range": "± 2697",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 232583129,
            "range": "± 1170042",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114604810,
            "range": "± 1090591",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10242217,
            "range": "± 157248",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36183075,
            "range": "± 53116",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48455813,
            "range": "± 339814",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14677128,
            "range": "± 9464",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2869930,
            "range": "± 18859",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2806816,
            "range": "± 2379",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "bc78af1c02758d055e797241f10d36daee3d8388",
          "message": "Provide context to initial build of subgraph, surface API",
          "timestamp": "2022-04-08T23:30:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/bc78af1c02758d055e797241f10d36daee3d8388"
        },
        "date": 1649908372602,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374938,
            "range": "± 2854",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 231728001,
            "range": "± 1824834",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117938848,
            "range": "± 1985951",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9998035,
            "range": "± 27321",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36570814,
            "range": "± 96021",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49315730,
            "range": "± 1077134",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691483,
            "range": "± 4564",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2715215,
            "range": "± 4730",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2708138,
            "range": "± 2845",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "bc78af1c02758d055e797241f10d36daee3d8388",
          "message": "Provide context to initial build of subgraph, surface API",
          "timestamp": "2022-04-08T23:30:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/bc78af1c02758d055e797241f10d36daee3d8388"
        },
        "date": 1649995392887,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 289775,
            "range": "± 8383",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 217423224,
            "range": "± 868020",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99901772,
            "range": "± 1156144",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10422902,
            "range": "± 157572",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34689345,
            "range": "± 111589",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57713471,
            "range": "± 98820",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18215231,
            "range": "± 17920",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3147206,
            "range": "± 5860",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2889599,
            "range": "± 3292",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b7ea31f28ee36e1324960ea48d5c1aa6abc19f93",
          "message": "Rename `into_parts()` to `make_parts()`",
          "timestamp": "2022-04-15T20:56:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b7ea31f28ee36e1324960ea48d5c1aa6abc19f93"
        },
        "date": 1650081133915,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 402533,
            "range": "± 27123",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 265834049,
            "range": "± 6999468",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 129332689,
            "range": "± 4872759",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14271679,
            "range": "± 649409",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42919545,
            "range": "± 2331072",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54243795,
            "range": "± 3245708",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 19557166,
            "range": "± 842478",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3607655,
            "range": "± 200778",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3533014,
            "range": "± 177384",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b7ea31f28ee36e1324960ea48d5c1aa6abc19f93",
          "message": "Rename `into_parts()` to `make_parts()`",
          "timestamp": "2022-04-15T20:56:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b7ea31f28ee36e1324960ea48d5c1aa6abc19f93"
        },
        "date": 1650167502026,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370959,
            "range": "± 5019",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 233878792,
            "range": "± 582778",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106412598,
            "range": "± 3881821",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10749888,
            "range": "± 56393",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35980156,
            "range": "± 41984",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48365757,
            "range": "± 406871",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681922,
            "range": "± 10355",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2714939,
            "range": "± 10518",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2672290,
            "range": "± 4092",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b7ea31f28ee36e1324960ea48d5c1aa6abc19f93",
          "message": "Rename `into_parts()` to `make_parts()`",
          "timestamp": "2022-04-15T20:56:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b7ea31f28ee36e1324960ea48d5c1aa6abc19f93"
        },
        "date": 1650254182865,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375828,
            "range": "± 2829",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 232506747,
            "range": "± 1434180",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117961232,
            "range": "± 1902848",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11363876,
            "range": "± 71914",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37032705,
            "range": "± 164730",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49306321,
            "range": "± 1069362",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687071,
            "range": "± 10707",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2684483,
            "range": "± 4877",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655122,
            "range": "± 7200",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b7ea31f28ee36e1324960ea48d5c1aa6abc19f93",
          "message": "Rename `into_parts()` to `make_parts()`",
          "timestamp": "2022-04-15T20:56:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b7ea31f28ee36e1324960ea48d5c1aa6abc19f93"
        },
        "date": 1650340869402,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376176,
            "range": "± 2757",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 236829178,
            "range": "± 8094901",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 118010046,
            "range": "± 1314112",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8987003,
            "range": "± 30476",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33876397,
            "range": "± 540898",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60189129,
            "range": "± 1685157",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12977933,
            "range": "± 9625",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2849854,
            "range": "± 58998",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2741668,
            "range": "± 47105",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b7ea31f28ee36e1324960ea48d5c1aa6abc19f93",
          "message": "Rename `into_parts()` to `make_parts()`",
          "timestamp": "2022-04-15T20:56:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b7ea31f28ee36e1324960ea48d5c1aa6abc19f93"
        },
        "date": 1650427427816,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371779,
            "range": "± 2973",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 222534431,
            "range": "± 345359",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 110076348,
            "range": "± 608501",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12611601,
            "range": "± 58756",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36232163,
            "range": "± 74390",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49010276,
            "range": "± 482125",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689313,
            "range": "± 7127",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2681793,
            "range": "± 3353",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2685178,
            "range": "± 4009",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650513873634,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 408767,
            "range": "± 8969",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 280172493,
            "range": "± 4920396",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 139880593,
            "range": "± 3859907",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14017327,
            "range": "± 597775",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43798830,
            "range": "± 1015722",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 68343945,
            "range": "± 1993899",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18948677,
            "range": "± 424870",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3697255,
            "range": "± 117092",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3598532,
            "range": "± 104746",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650600261911,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 330735,
            "range": "± 2116",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 214856761,
            "range": "± 1563109",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106606255,
            "range": "± 1663686",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10483091,
            "range": "± 92913",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32587169,
            "range": "± 77432",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41810076,
            "range": "± 433270",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12957372,
            "range": "± 36173",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2386976,
            "range": "± 7722",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2367715,
            "range": "± 2009",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650685972568,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 317500,
            "range": "± 4966",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216081229,
            "range": "± 738994",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98997625,
            "range": "± 1761340",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10822668,
            "range": "± 46708",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34344771,
            "range": "± 147687",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57504242,
            "range": "± 361158",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18214318,
            "range": "± 20151",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2926458,
            "range": "± 8322",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2883269,
            "range": "± 2444",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650772278091,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375100,
            "range": "± 3008",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 235830505,
            "range": "± 1455712",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109437371,
            "range": "± 1171965",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10072019,
            "range": "± 43662",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36508067,
            "range": "± 100844",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49812856,
            "range": "± 1457435",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682985,
            "range": "± 9322",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2699960,
            "range": "± 5734",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2657900,
            "range": "± 2359",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650859369844,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 409630,
            "range": "± 22702",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 280603753,
            "range": "± 7217946",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 137068714,
            "range": "± 5157696",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14284179,
            "range": "± 618289",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43573921,
            "range": "± 1723137",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 68847286,
            "range": "± 3435920",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 19209284,
            "range": "± 761028",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3785324,
            "range": "± 199311",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3732737,
            "range": "± 163985",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cf308c5efc303e210b02e0c377caa58f8a6a0a9b",
          "message": "Provide `&mut Context` to subgraph closure, instead of just ref\n\nFor now does not affect any external API, but is a preqrequisite for\n`&mut Context` in the surface API",
          "timestamp": "2022-04-19T23:51:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cf308c5efc303e210b02e0c377caa58f8a6a0a9b"
        },
        "date": 1650945560115,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376020,
            "range": "± 2812",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 234707393,
            "range": "± 2150142",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121571332,
            "range": "± 1704183",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10901789,
            "range": "± 184887",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37217228,
            "range": "± 279624",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52287302,
            "range": "± 1877724",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14697633,
            "range": "± 8141",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2645374,
            "range": "± 7497",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2686286,
            "range": "± 16012",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c55745dcc9fb838fd0bebcfb11a5a181918d19b8",
          "message": "Make KVS benchmark more Anna-like (#146)\n\nI decided it made the most sense to remove the other implementations here,\r\nsince there were a lot of structural changes going on, they made it annoying to\r\nupdate this. They live on in the repo history if they ever need to be revived.\r\n\r\nThis commit moves the workload generation into the workers themselves, since\r\notherwise I was unable to fully saturate the workers.",
          "timestamp": "2022-04-26T17:44:11Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c55745dcc9fb838fd0bebcfb11a5a181918d19b8"
        },
        "date": 1651032287126,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 383456,
            "range": "± 20791",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 234028200,
            "range": "± 8369251",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116668833,
            "range": "± 4321664",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12984377,
            "range": "± 735674",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40618452,
            "range": "± 1720109",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56886512,
            "range": "± 4342930",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16943452,
            "range": "± 988482",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3439759,
            "range": "± 175875",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3310761,
            "range": "± 150442",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5f4618ee81bd6a3bf9c0bbde509dc5b264a1f646",
          "message": "Add scheduling method to context, update docs\n\nIssue #143",
          "timestamp": "2022-04-21T22:22:31Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5f4618ee81bd6a3bf9c0bbde509dc5b264a1f646"
        },
        "date": 1651119761438,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375696,
            "range": "± 2363",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 220674725,
            "range": "± 1472084",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111032310,
            "range": "± 872375",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11118049,
            "range": "± 37711",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32478810,
            "range": "± 91419",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 42788472,
            "range": "± 475355",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685849,
            "range": "± 8112",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2430076,
            "range": "± 4605",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2349193,
            "range": "± 2016",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9",
          "message": "Improve docs in KVS (#147)",
          "timestamp": "2022-04-28T20:53:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9"
        },
        "date": 1651204890714,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 369075,
            "range": "± 19523",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 265656408,
            "range": "± 8764467",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 130112113,
            "range": "± 5251798",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12722016,
            "range": "± 482623",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39850903,
            "range": "± 1701329",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61651057,
            "range": "± 2651193",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16804690,
            "range": "± 955871",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3372460,
            "range": "± 150906",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3500371,
            "range": "± 221686",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9",
          "message": "Improve docs in KVS (#147)",
          "timestamp": "2022-04-28T20:53:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9"
        },
        "date": 1651290954475,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 260998,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216103199,
            "range": "± 817260",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98409757,
            "range": "± 418401",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10353010,
            "range": "± 93364",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34204609,
            "range": "± 117040",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57534576,
            "range": "± 100907",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209494,
            "range": "± 25301",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2912373,
            "range": "± 9003",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3013653,
            "range": "± 6812",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9",
          "message": "Improve docs in KVS (#147)",
          "timestamp": "2022-04-28T20:53:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9"
        },
        "date": 1651377753116,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370442,
            "range": "± 2642",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 231577527,
            "range": "± 1408545",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114031485,
            "range": "± 1364648",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10686194,
            "range": "± 145864",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36876426,
            "range": "± 51746",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47663853,
            "range": "± 177333",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14678604,
            "range": "± 10978",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2663623,
            "range": "± 16463",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2668255,
            "range": "± 2916",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9",
          "message": "Improve docs in KVS (#147)",
          "timestamp": "2022-04-28T20:53:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9"
        },
        "date": 1651464665549,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 311432,
            "range": "± 5731",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 218053867,
            "range": "± 1636246",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99681432,
            "range": "± 438332",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10989283,
            "range": "± 72235",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34967338,
            "range": "± 137624",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57100534,
            "range": "± 88785",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18215631,
            "range": "± 15965",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2969953,
            "range": "± 17240",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2993600,
            "range": "± 4442",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Justin Jaffray",
            "username": "justinj",
            "email": "justin.jaffray@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9",
          "message": "Improve docs in KVS (#147)",
          "timestamp": "2022-04-28T20:53:28Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/36886fc8d0f6f25f3b20c79b6f8613a8e81c2be9"
        },
        "date": 1651550422881,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375699,
            "range": "± 3255",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 231937833,
            "range": "± 2297204",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 115016815,
            "range": "± 1550269",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10584398,
            "range": "± 64307",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36286730,
            "range": "± 62586",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48127866,
            "range": "± 438363",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684682,
            "range": "± 76917",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2657036,
            "range": "± 5815",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2633417,
            "range": "± 1791",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1651636791124,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374798,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 209639770,
            "range": "± 1522547",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109826207,
            "range": "± 1519553",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10665071,
            "range": "± 114299",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36275046,
            "range": "± 69778",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59255572,
            "range": "± 370550",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682547,
            "range": "± 8090",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2639618,
            "range": "± 3974",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2633993,
            "range": "± 2065",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1651723347692,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 316278,
            "range": "± 6710",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 219943946,
            "range": "± 1056972",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 91870289,
            "range": "± 1506896",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10568740,
            "range": "± 151950",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34529495,
            "range": "± 107255",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76848422,
            "range": "± 252220",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18221824,
            "range": "± 10187",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2882240,
            "range": "± 9140",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2904029,
            "range": "± 1857",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1651809304407,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 386754,
            "range": "± 8615",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 245957556,
            "range": "± 3833969",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120182201,
            "range": "± 2524566",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14412049,
            "range": "± 402899",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40891759,
            "range": "± 1108124",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 75731813,
            "range": "± 2007687",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18524465,
            "range": "± 683703",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3609317,
            "range": "± 94499",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3461136,
            "range": "± 92209",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1651895722023,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 381286,
            "range": "± 12869",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 241100695,
            "range": "± 6409033",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119874643,
            "range": "± 5590880",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14127024,
            "range": "± 708215",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39248001,
            "range": "± 1451358",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 71788103,
            "range": "± 3901219",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17975643,
            "range": "± 1220969",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3337443,
            "range": "± 210050",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3399267,
            "range": "± 255909",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1651982193288,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 450433,
            "range": "± 3332",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 257522267,
            "range": "± 842185",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126607116,
            "range": "± 1900135",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14314721,
            "range": "± 130185",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 43141305,
            "range": "± 289305",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 69798385,
            "range": "± 648589",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17541259,
            "range": "± 148729",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3189216,
            "range": "± 74831",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3160256,
            "range": "± 28281",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1652068757305,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 262810,
            "range": "± 2950",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216696102,
            "range": "± 1034782",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99181150,
            "range": "± 1635562",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10571740,
            "range": "± 101408",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34460337,
            "range": "± 85344",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76994853,
            "range": "± 76908",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18208776,
            "range": "± 23271",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2831093,
            "range": "± 3409",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2887042,
            "range": "± 2307",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1652154717264,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 360406,
            "range": "± 17383",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 259442638,
            "range": "± 5118837",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 128628310,
            "range": "± 3017767",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13306546,
            "range": "± 433772",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39026157,
            "range": "± 1317598",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72267710,
            "range": "± 3117420",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16875982,
            "range": "± 616489",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3266279,
            "range": "± 176095",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3355498,
            "range": "± 153859",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1652241634672,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375863,
            "range": "± 3014",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 233480702,
            "range": "± 515796",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119669318,
            "range": "± 885937",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13018887,
            "range": "± 51659",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36243869,
            "range": "± 168558",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 63389355,
            "range": "± 1152198",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689922,
            "range": "± 7421",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2673541,
            "range": "± 33004",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2664208,
            "range": "± 4093",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1652327851135,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 345764,
            "range": "± 15158",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 212836095,
            "range": "± 6319815",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 107877835,
            "range": "± 4319374",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12758294,
            "range": "± 688810",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37131983,
            "range": "± 3062518",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 64975754,
            "range": "± 3543186",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16411478,
            "range": "± 839612",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3135013,
            "range": "± 165525",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3066006,
            "range": "± 178562",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "77ebfc348c3746d2af008c6f6d6eafa3ef99926e",
          "message": "Update dependencies in Cargo.lock",
          "timestamp": "2022-05-03T17:12:26Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/77ebfc348c3746d2af008c6f6d6eafa3ef99926e"
        },
        "date": 1652414855860,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375969,
            "range": "± 2554",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 234103868,
            "range": "± 1313791",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121182439,
            "range": "± 2504506",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13693485,
            "range": "± 94746",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36711930,
            "range": "± 121131",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61978099,
            "range": "± 1313332",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14692605,
            "range": "± 9267",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2662797,
            "range": "± 5930",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2675347,
            "range": "± 3727",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653019097124,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 316870,
            "range": "± 7436",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 218895255,
            "range": "± 1709948",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 95674808,
            "range": "± 1816402",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12868141,
            "range": "± 141797",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35481841,
            "range": "± 82836",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49503759,
            "range": "± 662864",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14674099,
            "range": "± 18767",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2645538,
            "range": "± 3985",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2286299,
            "range": "± 33089",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653105079973,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375606,
            "range": "± 3564",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 226281128,
            "range": "± 449647",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117117225,
            "range": "± 3218792",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12881805,
            "range": "± 24179",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36187847,
            "range": "± 443420",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60723643,
            "range": "± 1457893",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685131,
            "range": "± 12343",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2633308,
            "range": "± 3652",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2624389,
            "range": "± 13571",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653191673099,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375761,
            "range": "± 2636",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 224287679,
            "range": "± 1120382",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 115980117,
            "range": "± 1260813",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10642825,
            "range": "± 33937",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35994471,
            "range": "± 422134",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60793177,
            "range": "± 473401",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14676454,
            "range": "± 8944",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2654201,
            "range": "± 18347",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655969,
            "range": "± 4316",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653278516075,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 424372,
            "range": "± 14308",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 238318095,
            "range": "± 8721749",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 121611737,
            "range": "± 3480322",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12637431,
            "range": "± 597198",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42007267,
            "range": "± 896108",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72660619,
            "range": "± 880192",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16444414,
            "range": "± 873824",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2969861,
            "range": "± 120404",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2983692,
            "range": "± 150577",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653364912790,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 450781,
            "range": "± 3068",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 256128624,
            "range": "± 18747104",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123104959,
            "range": "± 3681162",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 16268649,
            "range": "± 1063673",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42424092,
            "range": "± 121319",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72703325,
            "range": "± 791901",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17633857,
            "range": "± 15183",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3160524,
            "range": "± 118714",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3319889,
            "range": "± 8720",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653451152303,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 264381,
            "range": "± 3961",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 221498436,
            "range": "± 2487537",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101155693,
            "range": "± 1936248",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11343437,
            "range": "± 327126",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35736276,
            "range": "± 1007103",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72127352,
            "range": "± 1226699",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18443515,
            "range": "± 384070",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3149826,
            "range": "± 91695",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3155423,
            "range": "± 40258",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653537632136,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261232,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 218073144,
            "range": "± 1678290",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99740686,
            "range": "± 1077956",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10918531,
            "range": "± 86661",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34210185,
            "range": "± 160977",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76749997,
            "range": "± 133855",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18223318,
            "range": "± 12238",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2901691,
            "range": "± 4918",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2978547,
            "range": "± 10673",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653624142846,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 388783,
            "range": "± 19083",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 252847019,
            "range": "± 8301256",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125627972,
            "range": "± 4592404",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14628154,
            "range": "± 806975",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41100228,
            "range": "± 1542640",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 77795346,
            "range": "± 4266726",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18569738,
            "range": "± 1334002",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3476155,
            "range": "± 165221",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3489389,
            "range": "± 112920",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653710221345,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 439488,
            "range": "± 7695",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 253177676,
            "range": "± 3107871",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123889053,
            "range": "± 1833073",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15900651,
            "range": "± 185893",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41429008,
            "range": "± 522509",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 70709404,
            "range": "± 900057",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17461485,
            "range": "± 213197",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3099450,
            "range": "± 32854",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3151474,
            "range": "± 41830",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653796883827,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 333857,
            "range": "± 33227",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 221501022,
            "range": "± 9279582",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125571099,
            "range": "± 6463013",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13297723,
            "range": "± 934026",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37183873,
            "range": "± 2321204",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 68638810,
            "range": "± 5103423",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16143083,
            "range": "± 1034569",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3277931,
            "range": "± 205151",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3250940,
            "range": "± 171433",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653883325144,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375603,
            "range": "± 2917",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 190268291,
            "range": "± 985691",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 112210368,
            "range": "± 1621550",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13529207,
            "range": "± 64692",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36176955,
            "range": "± 75253",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59054170,
            "range": "± 276478",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679668,
            "range": "± 11080",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2701287,
            "range": "± 11447",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2690793,
            "range": "± 2029",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1653969627177,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370562,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186412738,
            "range": "± 440838",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109693687,
            "range": "± 1179686",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13534522,
            "range": "± 68827",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36174984,
            "range": "± 88024",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59073719,
            "range": "± 442394",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14680517,
            "range": "± 10731",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2703464,
            "range": "± 3869",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2628339,
            "range": "± 2770",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654057354343,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261399,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168813118,
            "range": "± 1793025",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97770043,
            "range": "± 1632783",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10929782,
            "range": "± 77470",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34204413,
            "range": "± 145189",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 76968793,
            "range": "± 117911",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18210739,
            "range": "± 359948",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2962836,
            "range": "± 5058",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2853966,
            "range": "± 3520",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654142905897,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 363193,
            "range": "± 16466",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199272004,
            "range": "± 6254541",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114355120,
            "range": "± 4058321",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12897619,
            "range": "± 802499",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 40042357,
            "range": "± 2658445",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 72910821,
            "range": "± 3849440",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17159359,
            "range": "± 918753",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3259788,
            "range": "± 117998",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3357357,
            "range": "± 141112",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654228563558,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375637,
            "range": "± 2796",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187443914,
            "range": "± 1031759",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108185468,
            "range": "± 2323625",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10649152,
            "range": "± 38513",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36928702,
            "range": "± 222881",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59086210,
            "range": "± 1861434",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683436,
            "range": "± 21057",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2746835,
            "range": "± 30223",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655070,
            "range": "± 11893",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654314779097,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375629,
            "range": "± 20112",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199399423,
            "range": "± 1132223",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117834544,
            "range": "± 2141601",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11971468,
            "range": "± 116891",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35634052,
            "range": "± 69783",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60668808,
            "range": "± 1003768",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682243,
            "range": "± 12170",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2709020,
            "range": "± 17621",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2741997,
            "range": "± 3058",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654401167857,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375614,
            "range": "± 2744",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198106549,
            "range": "± 1409755",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114992938,
            "range": "± 1185716",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9937058,
            "range": "± 39560",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36102427,
            "range": "± 68470",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59741316,
            "range": "± 232636",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685997,
            "range": "± 9079",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2703456,
            "range": "± 6726",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2679227,
            "range": "± 3967",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654488060624,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 433153,
            "range": "± 32382",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 210958975,
            "range": "± 7040308",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116398457,
            "range": "± 4386607",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14779597,
            "range": "± 1107869",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41446745,
            "range": "± 844482",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 67791723,
            "range": "± 4405200",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17271216,
            "range": "± 1055285",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3165845,
            "range": "± 193413",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3213257,
            "range": "± 195175",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654574414029,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371291,
            "range": "± 2399",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198541169,
            "range": "± 665263",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116622461,
            "range": "± 1802734",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11996245,
            "range": "± 43803",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36072000,
            "range": "± 95205",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59598926,
            "range": "± 896962",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684391,
            "range": "± 40745",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2816708,
            "range": "± 6851",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2684780,
            "range": "± 2655",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654660888070,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 330557,
            "range": "± 3093",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 174191616,
            "range": "± 1307864",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100509919,
            "range": "± 1080189",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10071622,
            "range": "± 36777",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36305764,
            "range": "± 62475",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59783495,
            "range": "± 323766",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683515,
            "range": "± 8035",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2711035,
            "range": "± 4679",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2765652,
            "range": "± 18678",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654747246552,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261739,
            "range": "± 1218",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 170014400,
            "range": "± 1398587",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105740393,
            "range": "± 760676",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10454891,
            "range": "± 558966",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36928714,
            "range": "± 90396",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 77737638,
            "range": "± 339478",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18219771,
            "range": "± 23142",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3027139,
            "range": "± 4177",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3832456,
            "range": "± 4117",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654833564582,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 430310,
            "range": "± 18610",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 210541742,
            "range": "± 2559330",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 122609574,
            "range": "± 2034320",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11400039,
            "range": "± 305036",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 42164890,
            "range": "± 806104",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 68021897,
            "range": "± 2524820",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17271467,
            "range": "± 400289",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3253166,
            "range": "± 49523",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3195951,
            "range": "± 14117",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1654919821606,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375364,
            "range": "± 2956",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 202308269,
            "range": "± 919501",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120844835,
            "range": "± 1121988",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11328238,
            "range": "± 27223",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36658994,
            "range": "± 183515",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 63240575,
            "range": "± 1418927",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14692653,
            "range": "± 9555",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2722267,
            "range": "± 6011",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2661584,
            "range": "± 3561",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1655006319375,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 407361,
            "range": "± 17704",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 205055652,
            "range": "± 3618173",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 124184449,
            "range": "± 1760343",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11173115,
            "range": "± 519553",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41352796,
            "range": "± 1630656",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66899073,
            "range": "± 2750169",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16389480,
            "range": "± 604707",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2972236,
            "range": "± 102957",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3018337,
            "range": "± 88086",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1655093020131,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 225286,
            "range": "± 15987",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 151227283,
            "range": "± 2045803",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89831555,
            "range": "± 1565419",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8975569,
            "range": "± 68653",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 29582889,
            "range": "± 164069",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66563635,
            "range": "± 310719",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15714400,
            "range": "± 53537",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2580082,
            "range": "± 26791",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2509797,
            "range": "± 12088",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1655179741763,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375012,
            "range": "± 2805",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186936388,
            "range": "± 923121",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109979991,
            "range": "± 487070",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10038727,
            "range": "± 74132",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36222205,
            "range": "± 60325",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58718244,
            "range": "± 329928",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682140,
            "range": "± 7641",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2705792,
            "range": "± 7276",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2662250,
            "range": "± 1958",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "b6a942f6a282fc142080e234e3eb1c16b3fcc34d",
          "message": "Update SHJ to return `(K, (V1, V2))` for easier chaining (instead of `(K, V1, V2)`)",
          "timestamp": "2022-05-19T21:58:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b6a942f6a282fc142080e234e3eb1c16b3fcc34d"
        },
        "date": 1655265802500,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 317597,
            "range": "± 18960",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 210946515,
            "range": "± 11124547",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 117038229,
            "range": "± 5665164",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12383676,
            "range": "± 816997",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35321114,
            "range": "± 1924959",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 66313736,
            "range": "± 4863889",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16653680,
            "range": "± 878326",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3268202,
            "range": "± 187850",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3326105,
            "range": "± 175258",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655352047463,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375879,
            "range": "± 2796",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200304551,
            "range": "± 925888",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 118011501,
            "range": "± 1910722",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11266998,
            "range": "± 25960",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35820428,
            "range": "± 173374",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60046705,
            "range": "± 1184170",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689617,
            "range": "± 10612",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2566013,
            "range": "± 7941",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2465074,
            "range": "± 4966",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655438461592,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 295314,
            "range": "± 10341",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168534170,
            "range": "± 931668",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 102873646,
            "range": "± 1655200",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10125936,
            "range": "± 51704",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36724012,
            "range": "± 2200177",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 77306974,
            "range": "± 185187",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18225342,
            "range": "± 103015",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2941695,
            "range": "± 4748",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2938817,
            "range": "± 231258",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655524679606,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 345163,
            "range": "± 20112",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 191520519,
            "range": "± 4875920",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111068524,
            "range": "± 5291628",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10687534,
            "range": "± 403298",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37282905,
            "range": "± 1205397",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59139151,
            "range": "± 1835432",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14891175,
            "range": "± 646603",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2753306,
            "range": "± 104984",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2674719,
            "range": "± 96896",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655611302265,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370255,
            "range": "± 2172",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166295056,
            "range": "± 261981",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98117387,
            "range": "± 1940406",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10588882,
            "range": "± 31594",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32033750,
            "range": "± 35792",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38645036,
            "range": "± 143618",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14676442,
            "range": "± 11813",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2639090,
            "range": "± 3707",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2656906,
            "range": "± 1675",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655697475155,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261558,
            "range": "± 2171",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 147732050,
            "range": "± 711563",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90624210,
            "range": "± 1187662",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10826538,
            "range": "± 61694",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32708314,
            "range": "± 115282",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50061238,
            "range": "± 26399",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18205813,
            "range": "± 36076",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3809944,
            "range": "± 2773",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2977139,
            "range": "± 6170",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655784150532,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374680,
            "range": "± 2637",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 176470588,
            "range": "± 916331",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 94549629,
            "range": "± 1571001",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10682094,
            "range": "± 26883",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31148156,
            "range": "± 63776",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38327561,
            "range": "± 169861",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14676265,
            "range": "± 10784",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2639072,
            "range": "± 4006",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655664,
            "range": "± 2564",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655870490806,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 264535,
            "range": "± 3086",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 147133359,
            "range": "± 915620",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89993715,
            "range": "± 673461",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10420129,
            "range": "± 79510",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32919008,
            "range": "± 164838",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49541013,
            "range": "± 1216681",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224955,
            "range": "± 367282",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2867903,
            "range": "± 73355",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2951753,
            "range": "± 9137",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1655956925059,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 372466,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 167650857,
            "range": "± 1024509",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98894275,
            "range": "± 423254",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11233798,
            "range": "± 23004",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31627841,
            "range": "± 52086",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38996235,
            "range": "± 208762",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14686296,
            "range": "± 7563",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2608406,
            "range": "± 8092",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2635048,
            "range": "± 2912",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1656043400410,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375721,
            "range": "± 2731",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 177541314,
            "range": "± 4706876",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105238636,
            "range": "± 2916195",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12795325,
            "range": "± 63870",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31638049,
            "range": "± 58423",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38132424,
            "range": "± 342339",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685201,
            "range": "± 9461",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2659021,
            "range": "± 3753",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2641358,
            "range": "± 3801",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1656129660008,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 226069,
            "range": "± 7829",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 117459619,
            "range": "± 1822966",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 70050569,
            "range": "± 720927",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 7798673,
            "range": "± 90481",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 20434796,
            "range": "± 108747",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 36451124,
            "range": "± 177204",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 13018383,
            "range": "± 43839",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2078408,
            "range": "± 16197",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2103581,
            "range": "± 8327",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1656216084399,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374622,
            "range": "± 2792",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 178833767,
            "range": "± 601041",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105899502,
            "range": "± 1698027",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13195985,
            "range": "± 97024",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31846313,
            "range": "± 49743",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38905848,
            "range": "± 392920",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684874,
            "range": "± 97652",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2640188,
            "range": "± 5265",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2689547,
            "range": "± 3128",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "548373a247858beefe0848b4061f091ee124b472",
          "message": "Cleanup deref for clippy",
          "timestamp": "2022-06-15T20:33:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/548373a247858beefe0848b4061f091ee124b472"
        },
        "date": 1656302907242,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 319273,
            "range": "± 6822",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 139707135,
            "range": "± 1252971",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90193283,
            "range": "± 623850",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10443832,
            "range": "± 159090",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33199318,
            "range": "± 103472",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50314162,
            "range": "± 102259",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18208707,
            "range": "± 27323",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2991895,
            "range": "± 25158",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3084614,
            "range": "± 3814",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656389072354,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 275713,
            "range": "± 9171",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 161123062,
            "range": "± 2050498",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98300290,
            "range": "± 1484866",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10363691,
            "range": "± 98684",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32803403,
            "range": "± 103320",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50521415,
            "range": "± 107004",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224616,
            "range": "± 46787",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2843742,
            "range": "± 8150",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2919811,
            "range": "± 8868",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656475645828,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 393034,
            "range": "± 24028",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 223831639,
            "range": "± 6245250",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120747069,
            "range": "± 5626673",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14316265,
            "range": "± 1050914",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37322995,
            "range": "± 1821483",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57911782,
            "range": "± 3665272",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18904543,
            "range": "± 650846",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3630858,
            "range": "± 109597",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3696460,
            "range": "± 128739",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656561798415,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261316,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 150782459,
            "range": "± 1652520",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90059828,
            "range": "± 1834809",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10747114,
            "range": "± 108311",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30114500,
            "range": "± 106759",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50253638,
            "range": "± 154203",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18214397,
            "range": "± 21933",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2844666,
            "range": "± 9615",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2962017,
            "range": "± 4535",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656648690697,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 280500,
            "range": "± 8957",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 147000419,
            "range": "± 811119",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90566062,
            "range": "± 627266",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10296744,
            "range": "± 62174",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32461414,
            "range": "± 101143",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50525941,
            "range": "± 64541",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18219762,
            "range": "± 29280",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2871329,
            "range": "± 11275",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2903916,
            "range": "± 3350",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656734653564,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375620,
            "range": "± 2705",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165779935,
            "range": "± 354967",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97557172,
            "range": "± 1717477",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12839601,
            "range": "± 11434",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31824062,
            "range": "± 44217",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38048974,
            "range": "± 140204",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688266,
            "range": "± 7201",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2617681,
            "range": "± 4125",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2698349,
            "range": "± 3138",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656820835984,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331474,
            "range": "± 2638",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166277616,
            "range": "± 618069",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100011838,
            "range": "± 902727",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9626838,
            "range": "± 596725",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 29029466,
            "range": "± 94631",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39051550,
            "range": "± 517613",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684645,
            "range": "± 60156",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2649722,
            "range": "± 4954",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2695815,
            "range": "± 130270",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656907789846,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 372771,
            "range": "± 12365",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 181168151,
            "range": "± 7085574",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106422409,
            "range": "± 4981576",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13323807,
            "range": "± 513074",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34001294,
            "range": "± 2046710",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45861301,
            "range": "± 2143081",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17632477,
            "range": "± 1559822",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3267315,
            "range": "± 244723",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3350952,
            "range": "± 183240",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "0fe0f40dd49bcd1164032ea331f06c209de2ce16",
          "message": "Cleanup old code, add helpful comments",
          "timestamp": "2022-06-17T16:42:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16"
        },
        "date": 1656993813583,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374968,
            "range": "± 2667",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 177289890,
            "range": "± 920551",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 102587417,
            "range": "± 1507520",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10392958,
            "range": "± 122032",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31295379,
            "range": "± 119539",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38718877,
            "range": "± 522225",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681673,
            "range": "± 9271",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2610961,
            "range": "± 6324",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2650051,
            "range": "± 2890",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "70727c04fd6062b9e6c01799dd87f94bada19cd3",
          "message": "Fix unused method and complex type lints",
          "timestamp": "2022-06-29T18:34:59Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/70727c04fd6062b9e6c01799dd87f94bada19cd3"
        },
        "date": 1657080524250,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 380376,
            "range": "± 26535",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 223637309,
            "range": "± 5819991",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123727716,
            "range": "± 3882208",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13588988,
            "range": "± 752138",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36529646,
            "range": "± 1699434",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 53669598,
            "range": "± 3246879",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18037583,
            "range": "± 981137",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3899316,
            "range": "± 233955",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3525346,
            "range": "± 256460",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657166738895,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370339,
            "range": "± 2467",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 178903217,
            "range": "± 1152856",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105250337,
            "range": "± 1341776",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11161627,
            "range": "± 83128",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32129493,
            "range": "± 42753",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39135002,
            "range": "± 451679",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688153,
            "range": "± 11711",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2649377,
            "range": "± 17435",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2780256,
            "range": "± 2530",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657252964796,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 374685,
            "range": "± 3049",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 167634947,
            "range": "± 300607",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 95923262,
            "range": "± 1116539",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11066521,
            "range": "± 37020",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31974088,
            "range": "± 37587",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38460070,
            "range": "± 266523",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14664286,
            "range": "± 61138",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2630936,
            "range": "± 25611",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655971,
            "range": "± 2633",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657339063634,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 395812,
            "range": "± 12144",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216457665,
            "range": "± 5077901",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120877755,
            "range": "± 4395233",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13671096,
            "range": "± 497018",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36949754,
            "range": "± 1473578",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55711245,
            "range": "± 2953539",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18633382,
            "range": "± 527571",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3445393,
            "range": "± 180652",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3456106,
            "range": "± 154577",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657425664857,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 389527,
            "range": "± 26409",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216324218,
            "range": "± 14097441",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 118600827,
            "range": "± 3431547",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13729087,
            "range": "± 508748",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35452666,
            "range": "± 1933989",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52795162,
            "range": "± 2616383",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17668776,
            "range": "± 782988",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3265591,
            "range": "± 120900",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3378114,
            "range": "± 370314",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657512059466,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 328424,
            "range": "± 1779",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 157409540,
            "range": "± 871790",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 94047831,
            "range": "± 694034",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9127782,
            "range": "± 41820",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 28642857,
            "range": "± 68470",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 35747709,
            "range": "± 2340205",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12964056,
            "range": "± 9298",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2354278,
            "range": "± 14026",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2379131,
            "range": "± 6195",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657599295167,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375636,
            "range": "± 2435",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168374604,
            "range": "± 704740",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99505987,
            "range": "± 1141098",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10910977,
            "range": "± 60880",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31964155,
            "range": "± 48569",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38931678,
            "range": "± 222778",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14683318,
            "range": "± 10916",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2664278,
            "range": "± 4474",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2726972,
            "range": "± 3298",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657684936292,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331575,
            "range": "± 2962",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 157838916,
            "range": "± 2134405",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99406692,
            "range": "± 431808",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11707532,
            "range": "± 75041",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 28547865,
            "range": "± 56638",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 34799203,
            "range": "± 275375",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12961727,
            "range": "± 8024",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2635176,
            "range": "± 5981",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2396623,
            "range": "± 2393",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657771696435,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 438430,
            "range": "± 8568",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187416907,
            "range": "± 1233966",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111194638,
            "range": "± 982778",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12782753,
            "range": "± 160748",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37507953,
            "range": "± 210371",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45969669,
            "range": "± 770150",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17150381,
            "range": "± 236163",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3068090,
            "range": "± 59732",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3122793,
            "range": "± 48779",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20382f13d9baf49ee896a6c643bb25788aff2db0",
          "message": "Add #![allow(clippy::explicit_auto_deref)] due to false positives\n\nhttps://github.com/rust-lang/rust-clippy/issues/9101",
          "timestamp": "2022-07-06T20:24:38Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0"
        },
        "date": 1657858471731,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375587,
            "range": "± 2439",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165503710,
            "range": "± 1120360",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96071135,
            "range": "± 460255",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10826694,
            "range": "± 53833",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31927137,
            "range": "± 47210",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38275818,
            "range": "± 92177",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14674519,
            "range": "± 10187",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2657833,
            "range": "± 4037",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2668688,
            "range": "± 2696",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1657944006893,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 392716,
            "range": "± 177614",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 215461663,
            "range": "± 4967069",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 119075200,
            "range": "± 5531420",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13183480,
            "range": "± 624893",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36902408,
            "range": "± 1335327",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 53709266,
            "range": "± 3109182",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16877984,
            "range": "± 918790",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3304789,
            "range": "± 179353",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3397211,
            "range": "± 335562",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658030504838,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375683,
            "range": "± 3012",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 167258993,
            "range": "± 686614",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90772039,
            "range": "± 1463515",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10825258,
            "range": "± 41411",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32484178,
            "range": "± 104103",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38666216,
            "range": "± 301455",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687803,
            "range": "± 11358",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2612778,
            "range": "± 35628",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2663573,
            "range": "± 3895",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658117218278,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 359932,
            "range": "± 19899",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199716089,
            "range": "± 8358659",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105114257,
            "range": "± 4142577",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12809329,
            "range": "± 581664",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31576883,
            "range": "± 1630419",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 46534897,
            "range": "± 2496984",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16818756,
            "range": "± 757350",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3033190,
            "range": "± 137529",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3158801,
            "range": "± 144592",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658204215256,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 395820,
            "range": "± 26623",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 225366858,
            "range": "± 5650024",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 128884730,
            "range": "± 6177989",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14036729,
            "range": "± 848714",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38055050,
            "range": "± 2311574",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57777609,
            "range": "± 2969503",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18605427,
            "range": "± 752276",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3521604,
            "range": "± 206176",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3590147,
            "range": "± 173166",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658289843229,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 386475,
            "range": "± 30957",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 217751961,
            "range": "± 5626736",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125815870,
            "range": "± 5255894",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14764747,
            "range": "± 554447",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37182222,
            "range": "± 1323417",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54912485,
            "range": "± 2865088",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18425519,
            "range": "± 911525",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3498076,
            "range": "± 181509",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3559616,
            "range": "± 190476",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658376253012,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 397751,
            "range": "± 22143",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 224094702,
            "range": "± 7633903",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 127851875,
            "range": "± 4997406",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14796690,
            "range": "± 892503",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37501553,
            "range": "± 1808161",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58295823,
            "range": "± 3578283",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18718576,
            "range": "± 814152",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3503983,
            "range": "± 222540",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3650905,
            "range": "± 222192",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658462968193,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 385904,
            "range": "± 14402",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198304688,
            "range": "± 5508644",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 110193676,
            "range": "± 4179678",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13698433,
            "range": "± 530086",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34550165,
            "range": "± 1327212",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49797107,
            "range": "± 2818161",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17791466,
            "range": "± 672053",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3348483,
            "range": "± 261933",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3450692,
            "range": "± 120728",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658548841885,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371887,
            "range": "± 3991",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 179528687,
            "range": "± 1719942",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108828753,
            "range": "± 3654932",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10793035,
            "range": "± 29550",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31510344,
            "range": "± 157902",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39076695,
            "range": "± 508363",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687533,
            "range": "± 382103",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2714396,
            "range": "± 5505",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2718049,
            "range": "± 13078",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658635321610,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 385201,
            "range": "± 24376",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 194125864,
            "range": "± 10688337",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 102139217,
            "range": "± 6670660",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12778770,
            "range": "± 535232",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32589961,
            "range": "± 1261671",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47455676,
            "range": "± 1843570",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17196060,
            "range": "± 714135",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3226978,
            "range": "± 138383",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3205976,
            "range": "± 121656",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "5474e877fc895367e8401521666043fe8c027dc2",
          "message": "Modularize ops, provide nicer arity errors and now warnings too.",
          "timestamp": "2022-07-11T21:45:16Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5474e877fc895367e8401521666043fe8c027dc2"
        },
        "date": 1658722070972,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 359603,
            "range": "± 20987",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 200982666,
            "range": "± 8622544",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 115901427,
            "range": "± 8348877",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12579585,
            "range": "± 979384",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34997937,
            "range": "± 1475520",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49399431,
            "range": "± 3393149",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16272333,
            "range": "± 1105979",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3319222,
            "range": "± 250214",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3220102,
            "range": "± 219538",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b89c1976a180bcf18660ccde817d662791353470",
          "message": "clean up transitive closure test (#160)\n\n* clean up transitive closure test\r\n\r\n* simplify the join",
          "timestamp": "2022-07-26T00:50:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b89c1976a180bcf18660ccde817d662791353470"
        },
        "date": 1658809223845,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 292110,
            "range": "± 8305",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 146909364,
            "range": "± 1950550",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 87211763,
            "range": "± 1995195",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10811753,
            "range": "± 111138",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33254429,
            "range": "± 204269",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50487879,
            "range": "± 304104",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18218071,
            "range": "± 142970",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2863115,
            "range": "± 34365",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2888373,
            "range": "± 22868",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b89c1976a180bcf18660ccde817d662791353470",
          "message": "clean up transitive closure test (#160)\n\n* clean up transitive closure test\r\n\r\n* simplify the join",
          "timestamp": "2022-07-26T00:50:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b89c1976a180bcf18660ccde817d662791353470"
        },
        "date": 1658894913585,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 343594,
            "range": "± 20192",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 199200366,
            "range": "± 7767837",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114720642,
            "range": "± 3625996",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11807506,
            "range": "± 775592",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30429415,
            "range": "± 1531073",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45416703,
            "range": "± 3073774",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16511522,
            "range": "± 1067744",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2939537,
            "range": "± 162316",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2908075,
            "range": "± 134885",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "20c3eeb6e6b653e92277c35a759c320166693404",
          "message": "Check operator number of expression arguments",
          "timestamp": "2022-07-27T19:52:27Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/20c3eeb6e6b653e92277c35a759c320166693404"
        },
        "date": 1658981018834,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261467,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 145685778,
            "range": "± 955406",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 88545084,
            "range": "± 1362023",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11439092,
            "range": "± 59253",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33037703,
            "range": "± 21750",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50347661,
            "range": "± 44502",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18207821,
            "range": "± 20497",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2888681,
            "range": "± 5140",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2972300,
            "range": "± 2634",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "700268234d110c228c5e89c489ef698840863066",
          "message": "initial docs on surface syntax (#163)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls",
          "timestamp": "2022-07-28T21:47:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/700268234d110c228c5e89c489ef698840863066"
        },
        "date": 1659067752727,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 387851,
            "range": "± 12247",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 189080207,
            "range": "± 5364724",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108489681,
            "range": "± 3071028",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13531528,
            "range": "± 318557",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34672242,
            "range": "± 732076",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51317299,
            "range": "± 3344800",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18327690,
            "range": "± 1027022",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3426632,
            "range": "± 207803",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3416101,
            "range": "± 212860",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c1893dfdce96d31a3dc15626e0087ac7380403a5",
          "message": "document operators in the book (#164)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls\r\n\r\n* document ops\r\n\r\n* minor mods to book",
          "timestamp": "2022-07-29T19:51:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c1893dfdce96d31a3dc15626e0087ac7380403a5"
        },
        "date": 1659153987244,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 407677,
            "range": "± 21546",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 216899300,
            "range": "± 5991677",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 120376087,
            "range": "± 4111038",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14220251,
            "range": "± 387614",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37241722,
            "range": "± 1445302",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55566695,
            "range": "± 1433677",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18876086,
            "range": "± 593166",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3586061,
            "range": "± 155145",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3916370,
            "range": "± 216507",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c1893dfdce96d31a3dc15626e0087ac7380403a5",
          "message": "document operators in the book (#164)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls\r\n\r\n* document ops\r\n\r\n* minor mods to book",
          "timestamp": "2022-07-29T19:51:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c1893dfdce96d31a3dc15626e0087ac7380403a5"
        },
        "date": 1659240154840,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375886,
            "range": "± 2736",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168989369,
            "range": "± 784547",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101941712,
            "range": "± 1450011",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10987654,
            "range": "± 49978",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31416197,
            "range": "± 71853",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40383051,
            "range": "± 575480",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14696265,
            "range": "± 6471",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2658491,
            "range": "± 3213",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2661175,
            "range": "± 1823",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c1893dfdce96d31a3dc15626e0087ac7380403a5",
          "message": "document operators in the book (#164)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls\r\n\r\n* document ops\r\n\r\n* minor mods to book",
          "timestamp": "2022-07-29T19:51:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c1893dfdce96d31a3dc15626e0087ac7380403a5"
        },
        "date": 1659327769201,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375626,
            "range": "± 2719",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 174444648,
            "range": "± 1730777",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101838446,
            "range": "± 1438011",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10302431,
            "range": "± 66456",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31192022,
            "range": "± 75048",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38580892,
            "range": "± 696907",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689145,
            "range": "± 7818",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2641220,
            "range": "± 4500",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2630094,
            "range": "± 11263",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "db6c16f329ef47f49b488c1f99dd95ffcf0ec59d",
          "message": "full pass of book with surface syntax (#165)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls\r\n\r\n* document ops\r\n\r\n* minor mods to book\r\n\r\n* full pass of book with surface syntax",
          "timestamp": "2022-08-02T04:02:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/db6c16f329ef47f49b488c1f99dd95ffcf0ec59d"
        },
        "date": 1659414077796,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375712,
            "range": "± 5990",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 178986814,
            "range": "± 1457586",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101824590,
            "range": "± 1318885",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11363275,
            "range": "± 40898",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31076536,
            "range": "± 3634357",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 42108436,
            "range": "± 1703778",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691340,
            "range": "± 7277",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2634942,
            "range": "± 26331",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2654310,
            "range": "± 2865",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "db6c16f329ef47f49b488c1f99dd95ffcf0ec59d",
          "message": "full pass of book with surface syntax (#165)\n\n* clean up transitive closure test\r\n\r\n* simplify the join\r\n\r\n* SerdeGraph: serialize in parser, deserialize and plot in clients\r\n\r\n* lint and tests\r\n\r\n* code review cleanup\r\n\r\n* initial surface syntax docs\r\n\r\n* fix to_mermaid() calls\r\n\r\n* document ops\r\n\r\n* minor mods to book\r\n\r\n* full pass of book with surface syntax",
          "timestamp": "2022-08-02T04:02:39Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/db6c16f329ef47f49b488c1f99dd95ffcf0ec59d"
        },
        "date": 1659500227813,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375924,
            "range": "± 3259",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 177119597,
            "range": "± 2062050",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106147240,
            "range": "± 1255704",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10935525,
            "range": "± 88851",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31702054,
            "range": "± 143948",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41274521,
            "range": "± 935542",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14689718,
            "range": "± 9051",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2649961,
            "range": "± 4705",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2655937,
            "range": "± 3658",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1659585793894,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 448711,
            "range": "± 5335",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188205410,
            "range": "± 1964996",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111192064,
            "range": "± 2396208",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13508517,
            "range": "± 129206",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37834019,
            "range": "± 345595",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45725479,
            "range": "± 686022",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17454152,
            "range": "± 200692",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3150846,
            "range": "± 34838",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3179918,
            "range": "± 17039",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1659672320070,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 277236,
            "range": "± 9144",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 138220350,
            "range": "± 1351548",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90277411,
            "range": "± 865051",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10674335,
            "range": "± 64176",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33408343,
            "range": "± 71430",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50501039,
            "range": "± 145227",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18219941,
            "range": "± 22982",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2856046,
            "range": "± 6572",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2893399,
            "range": "± 2553",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1659758236802,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261667,
            "range": "± 1178",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 150202597,
            "range": "± 1474248",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 92821156,
            "range": "± 2034061",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11390148,
            "range": "± 137228",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33094216,
            "range": "± 82005",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50824549,
            "range": "± 436566",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18226914,
            "range": "± 11389",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2887019,
            "range": "± 16903",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2918678,
            "range": "± 3212",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1659844677048,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261503,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 138345097,
            "range": "± 830613",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 88642609,
            "range": "± 761597",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10616200,
            "range": "± 70965",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32981413,
            "range": "± 26741",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50380998,
            "range": "± 107962",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18214860,
            "range": "± 21397",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2914970,
            "range": "± 2197",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2937073,
            "range": "± 2620",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1659931355518,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261722,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 156033454,
            "range": "± 5299008",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 95621580,
            "range": "± 2584891",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11320641,
            "range": "± 79776",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33213542,
            "range": "± 132863",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51337528,
            "range": "± 500223",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18229252,
            "range": "± 12566",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2854392,
            "range": "± 19305",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2920737,
            "range": "± 26610",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1660017818283,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 319296,
            "range": "± 3275",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 148886412,
            "range": "± 799276",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89797507,
            "range": "± 1023851",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10666250,
            "range": "± 67923",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33181399,
            "range": "± 36845",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50255507,
            "range": "± 930127",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209203,
            "range": "± 20950",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3001665,
            "range": "± 148237",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2990035,
            "range": "± 16568",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1660103606861,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371759,
            "range": "± 3905",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 176662102,
            "range": "± 1420928",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 110379889,
            "range": "± 2150675",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10308157,
            "range": "± 36454",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31799310,
            "range": "± 211590",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41339882,
            "range": "± 1351150",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14696055,
            "range": "± 12319",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2664292,
            "range": "± 13724",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2763374,
            "range": "± 4650",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "43a5a8f528549067905783116ea75d8454944010",
          "message": "add echo server test",
          "timestamp": "2022-08-03T22:26:42Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/43a5a8f528549067905783116ea75d8454944010"
        },
        "date": 1660190404827,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 415884,
            "range": "± 30359",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 227611656,
            "range": "± 6536986",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 131399335,
            "range": "± 4784695",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14314725,
            "range": "± 668774",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38201943,
            "range": "± 1829870",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59233928,
            "range": "± 3368071",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 19078503,
            "range": "± 1026511",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3597402,
            "range": "± 214702",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3985778,
            "range": "± 226852",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660276862899,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375634,
            "range": "± 3849",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 177105414,
            "range": "± 1978318",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100184640,
            "range": "± 1673933",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10875050,
            "range": "± 51655",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31113355,
            "range": "± 195704",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38396112,
            "range": "± 141877",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14672323,
            "range": "± 14693",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2601094,
            "range": "± 3290",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2685527,
            "range": "± 15466",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660363077187,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375727,
            "range": "± 2827",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 163056033,
            "range": "± 2366396",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100086346,
            "range": "± 1873829",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11180694,
            "range": "± 48560",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30150598,
            "range": "± 95170",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37823227,
            "range": "± 714370",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684384,
            "range": "± 9020",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2557121,
            "range": "± 16767",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2604178,
            "range": "± 2463",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660449679380,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451010,
            "range": "± 6909",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 169600886,
            "range": "± 1788049",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105871216,
            "range": "± 1735122",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14045449,
            "range": "± 33697",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37114350,
            "range": "± 115202",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 46395904,
            "range": "± 608457",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17635513,
            "range": "± 17910",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3044814,
            "range": "± 7312",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3112251,
            "range": "± 14697",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660536880560,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 440272,
            "range": "± 6784",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 174357266,
            "range": "± 2115203",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 107426273,
            "range": "± 2269679",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13037723,
            "range": "± 307898",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35788401,
            "range": "± 638840",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 31410783,
            "range": "± 583063",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17191730,
            "range": "± 353100",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2949457,
            "range": "± 54679",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3038034,
            "range": "± 49151",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660622717007,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376062,
            "range": "± 7565",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165543164,
            "range": "± 2961010",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111257548,
            "range": "± 4852093",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10470933,
            "range": "± 60935",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31492258,
            "range": "± 251071",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38838266,
            "range": "± 1071370",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685346,
            "range": "± 40975",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2554418,
            "range": "± 6475",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2574476,
            "range": "± 5267",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "152d377d5f0c8c67bf2786ea17e54ccac272872a",
          "message": "Include mdbook tests directly",
          "timestamp": "2022-08-10T19:16:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/152d377d5f0c8c67bf2786ea17e54ccac272872a"
        },
        "date": 1660709807832,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 367223,
            "range": "± 10669",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 163029582,
            "range": "± 4222309",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 103446005,
            "range": "± 4670374",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12557546,
            "range": "± 656514",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32085978,
            "range": "± 1381866",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47002047,
            "range": "± 2051796",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17122763,
            "range": "± 768422",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3053245,
            "range": "± 121408",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3171868,
            "range": "± 164850",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f76dba1512e2e1c33e94c73e223fd30fb94f059",
          "message": "Add stratum consolidation as an optimization",
          "timestamp": "2022-08-17T22:55:21Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f76dba1512e2e1c33e94c73e223fd30fb94f059"
        },
        "date": 1660796231443,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375699,
            "range": "± 2568",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 153429993,
            "range": "± 1513700",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96472645,
            "range": "± 1200233",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10779725,
            "range": "± 97773",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30920012,
            "range": "± 406664",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38154497,
            "range": "± 285727",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14685660,
            "range": "± 9237",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2757626,
            "range": "± 6774",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2784023,
            "range": "± 16054",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7f76dba1512e2e1c33e94c73e223fd30fb94f059",
          "message": "Add stratum consolidation as an optimization",
          "timestamp": "2022-08-17T22:55:21Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7f76dba1512e2e1c33e94c73e223fd30fb94f059"
        },
        "date": 1660882194393,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370628,
            "range": "± 22403",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 192365170,
            "range": "± 5120839",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 115450554,
            "range": "± 5031476",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12693007,
            "range": "± 716231",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32223116,
            "range": "± 1549807",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48286074,
            "range": "± 2756149",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16352035,
            "range": "± 840045",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3326241,
            "range": "± 184982",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3383165,
            "range": "± 200845",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "76e9b5d563293b16b64f9ccaf8f373c92b5d5771",
          "message": "Run clippy with `--all-targets`",
          "timestamp": "2022-08-19T21:57:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/76e9b5d563293b16b64f9ccaf8f373c92b5d5771"
        },
        "date": 1660968144182,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 350605,
            "range": "± 27419",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 184170375,
            "range": "± 8832354",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113942173,
            "range": "± 6237900",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12904348,
            "range": "± 978998",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31470752,
            "range": "± 2981594",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45666198,
            "range": "± 3678480",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15921339,
            "range": "± 1291474",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3140282,
            "range": "± 213234",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3292722,
            "range": "± 240396",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "76e9b5d563293b16b64f9ccaf8f373c92b5d5771",
          "message": "Run clippy with `--all-targets`",
          "timestamp": "2022-08-19T21:57:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/76e9b5d563293b16b64f9ccaf8f373c92b5d5771"
        },
        "date": 1661054662248,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 387302,
            "range": "± 19083",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 175802169,
            "range": "± 8568504",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 112939294,
            "range": "± 6267699",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15137903,
            "range": "± 1206873",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36149254,
            "range": "± 2481643",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41940529,
            "range": "± 3411769",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18302463,
            "range": "± 1160985",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3514910,
            "range": "± 281421",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3536115,
            "range": "± 295684",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "76e9b5d563293b16b64f9ccaf8f373c92b5d5771",
          "message": "Run clippy with `--all-targets`",
          "timestamp": "2022-08-19T21:57:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/76e9b5d563293b16b64f9ccaf8f373c92b5d5771"
        },
        "date": 1661141934911,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 382000,
            "range": "± 18316",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166181179,
            "range": "± 5302361",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106747783,
            "range": "± 4538679",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14175887,
            "range": "± 912787",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34629046,
            "range": "± 1397718",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51688732,
            "range": "± 3795046",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18193386,
            "range": "± 1049651",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3498950,
            "range": "± 197348",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3460596,
            "range": "± 137055",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "76e9b5d563293b16b64f9ccaf8f373c92b5d5771",
          "message": "Run clippy with `--all-targets`",
          "timestamp": "2022-08-19T21:57:02Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/76e9b5d563293b16b64f9ccaf8f373c92b5d5771"
        },
        "date": 1661228805534,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378383,
            "range": "± 2729",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 152215111,
            "range": "± 1657320",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96404205,
            "range": "± 1176924",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10642121,
            "range": "± 147964",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32637661,
            "range": "± 33036",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38294690,
            "range": "± 102042",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14686359,
            "range": "± 4509",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2756222,
            "range": "± 17670",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2797654,
            "range": "± 20675",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "cca822a0f34b7ffe272ad50dde87d873743233c7",
          "message": "Use `BTreeMap` instead of `HashMap` in surface syntax codegen for determinism",
          "timestamp": "2022-08-19T22:25:08Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/cca822a0f34b7ffe272ad50dde87d873743233c7"
        },
        "date": 1661315075155,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 393593,
            "range": "± 71033",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 195771124,
            "range": "± 4320408",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 128388164,
            "range": "± 4318328",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14821748,
            "range": "± 524346",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35839628,
            "range": "± 1289934",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 55806852,
            "range": "± 3489103",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18752316,
            "range": "± 703427",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3684756,
            "range": "± 196040",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3704867,
            "range": "± 213313",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fd3867fde4302aabd747ca81564dfba6016a6395",
          "message": "Add datalog frontend via a proc macro (#155)\n\n* Implement parser for datalog and set up proc macro infra\r\n\r\n* Eliminate boxing of fields in Datalog grammar\r\n\r\n* Allow Souffle style trailing dot\r\n\r\n* Update rust-sitter\r\n\r\n* Initial graph creation logic\r\n\r\n* Update snapshots\r\n\r\n* Update to latest main\r\n\r\n* cargo fmt\r\n\r\n* Initial hacked up join implementation\r\n\r\n* Properly handle target bindings\r\n\r\n* Fix rules without a join\r\n\r\n* Move grammar to separate file\r\n\r\n* Add tees so that relations can be used multiple times\r\n\r\n* Fix other tests\r\n\r\n* Tee all relations\r\n\r\n* Support multiple contributors to one relation\r\n\r\n* Add transitive closure test\r\n\r\n* Support single column relations\r\n\r\n* Emit outputs to mpsc\r\n\r\n* Don't augment input/output names\r\n\r\n* Expand transitive closure test\r\n\r\n* Add join-with-self test that requires deterministic codegen\r\n\r\n* Address feedback\r\n\r\n* Extract join generation logic to a separate function\r\n\r\n* Eliminate assumption of usize columns\r\n\r\n* Rename datalog_compiler => hydroflow_datalog",
          "timestamp": "2022-08-24T05:53:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fd3867fde4302aabd747ca81564dfba6016a6395"
        },
        "date": 1661401670602,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376041,
            "range": "± 3284",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 179618791,
            "range": "± 1982013",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98739115,
            "range": "± 1715606",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12383548,
            "range": "± 209913",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31731146,
            "range": "± 2999342",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38932776,
            "range": "± 597805",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688230,
            "range": "± 9699",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2727751,
            "range": "± 43029",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2779360,
            "range": "± 35599",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fd3867fde4302aabd747ca81564dfba6016a6395",
          "message": "Add datalog frontend via a proc macro (#155)\n\n* Implement parser for datalog and set up proc macro infra\r\n\r\n* Eliminate boxing of fields in Datalog grammar\r\n\r\n* Allow Souffle style trailing dot\r\n\r\n* Update rust-sitter\r\n\r\n* Initial graph creation logic\r\n\r\n* Update snapshots\r\n\r\n* Update to latest main\r\n\r\n* cargo fmt\r\n\r\n* Initial hacked up join implementation\r\n\r\n* Properly handle target bindings\r\n\r\n* Fix rules without a join\r\n\r\n* Move grammar to separate file\r\n\r\n* Add tees so that relations can be used multiple times\r\n\r\n* Fix other tests\r\n\r\n* Tee all relations\r\n\r\n* Support multiple contributors to one relation\r\n\r\n* Add transitive closure test\r\n\r\n* Support single column relations\r\n\r\n* Emit outputs to mpsc\r\n\r\n* Don't augment input/output names\r\n\r\n* Expand transitive closure test\r\n\r\n* Add join-with-self test that requires deterministic codegen\r\n\r\n* Address feedback\r\n\r\n* Extract join generation logic to a separate function\r\n\r\n* Eliminate assumption of usize columns\r\n\r\n* Rename datalog_compiler => hydroflow_datalog",
          "timestamp": "2022-08-24T05:53:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fd3867fde4302aabd747ca81564dfba6016a6395"
        },
        "date": 1661488366057,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371484,
            "range": "± 13991",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186424508,
            "range": "± 5828918",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104981649,
            "range": "± 3946582",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14084985,
            "range": "± 822376",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32837316,
            "range": "± 1502796",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 46487668,
            "range": "± 1990437",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17368254,
            "range": "± 1065257",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3274563,
            "range": "± 167716",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3311417,
            "range": "± 142355",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fd3867fde4302aabd747ca81564dfba6016a6395",
          "message": "Add datalog frontend via a proc macro (#155)\n\n* Implement parser for datalog and set up proc macro infra\r\n\r\n* Eliminate boxing of fields in Datalog grammar\r\n\r\n* Allow Souffle style trailing dot\r\n\r\n* Update rust-sitter\r\n\r\n* Initial graph creation logic\r\n\r\n* Update snapshots\r\n\r\n* Update to latest main\r\n\r\n* cargo fmt\r\n\r\n* Initial hacked up join implementation\r\n\r\n* Properly handle target bindings\r\n\r\n* Fix rules without a join\r\n\r\n* Move grammar to separate file\r\n\r\n* Add tees so that relations can be used multiple times\r\n\r\n* Fix other tests\r\n\r\n* Tee all relations\r\n\r\n* Support multiple contributors to one relation\r\n\r\n* Add transitive closure test\r\n\r\n* Support single column relations\r\n\r\n* Emit outputs to mpsc\r\n\r\n* Don't augment input/output names\r\n\r\n* Expand transitive closure test\r\n\r\n* Add join-with-self test that requires deterministic codegen\r\n\r\n* Address feedback\r\n\r\n* Extract join generation logic to a separate function\r\n\r\n* Eliminate assumption of usize columns\r\n\r\n* Rename datalog_compiler => hydroflow_datalog",
          "timestamp": "2022-08-24T05:53:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fd3867fde4302aabd747ca81564dfba6016a6395"
        },
        "date": 1661573824882,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375870,
            "range": "± 2656",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 184928885,
            "range": "± 1234537",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113830730,
            "range": "± 2006626",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11306186,
            "range": "± 227897",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32116957,
            "range": "± 163020",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 44568842,
            "range": "± 3328672",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14700129,
            "range": "± 9870",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2773347,
            "range": "± 23414",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2796407,
            "range": "± 26664",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fd3867fde4302aabd747ca81564dfba6016a6395",
          "message": "Add datalog frontend via a proc macro (#155)\n\n* Implement parser for datalog and set up proc macro infra\r\n\r\n* Eliminate boxing of fields in Datalog grammar\r\n\r\n* Allow Souffle style trailing dot\r\n\r\n* Update rust-sitter\r\n\r\n* Initial graph creation logic\r\n\r\n* Update snapshots\r\n\r\n* Update to latest main\r\n\r\n* cargo fmt\r\n\r\n* Initial hacked up join implementation\r\n\r\n* Properly handle target bindings\r\n\r\n* Fix rules without a join\r\n\r\n* Move grammar to separate file\r\n\r\n* Add tees so that relations can be used multiple times\r\n\r\n* Fix other tests\r\n\r\n* Tee all relations\r\n\r\n* Support multiple contributors to one relation\r\n\r\n* Add transitive closure test\r\n\r\n* Support single column relations\r\n\r\n* Emit outputs to mpsc\r\n\r\n* Don't augment input/output names\r\n\r\n* Expand transitive closure test\r\n\r\n* Add join-with-self test that requires deterministic codegen\r\n\r\n* Address feedback\r\n\r\n* Extract join generation logic to a separate function\r\n\r\n* Eliminate assumption of usize columns\r\n\r\n* Rename datalog_compiler => hydroflow_datalog",
          "timestamp": "2022-08-24T05:53:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fd3867fde4302aabd747ca81564dfba6016a6395"
        },
        "date": 1661659936178,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375794,
            "range": "± 2517",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166473951,
            "range": "± 2012151",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101587360,
            "range": "± 1350693",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10705761,
            "range": "± 90735",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31235080,
            "range": "± 167759",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39372078,
            "range": "± 622600",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14692533,
            "range": "± 6111",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2735871,
            "range": "± 13966",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2790783,
            "range": "± 28475",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fd3867fde4302aabd747ca81564dfba6016a6395",
          "message": "Add datalog frontend via a proc macro (#155)\n\n* Implement parser for datalog and set up proc macro infra\r\n\r\n* Eliminate boxing of fields in Datalog grammar\r\n\r\n* Allow Souffle style trailing dot\r\n\r\n* Update rust-sitter\r\n\r\n* Initial graph creation logic\r\n\r\n* Update snapshots\r\n\r\n* Update to latest main\r\n\r\n* cargo fmt\r\n\r\n* Initial hacked up join implementation\r\n\r\n* Properly handle target bindings\r\n\r\n* Fix rules without a join\r\n\r\n* Move grammar to separate file\r\n\r\n* Add tees so that relations can be used multiple times\r\n\r\n* Fix other tests\r\n\r\n* Tee all relations\r\n\r\n* Support multiple contributors to one relation\r\n\r\n* Add transitive closure test\r\n\r\n* Support single column relations\r\n\r\n* Emit outputs to mpsc\r\n\r\n* Don't augment input/output names\r\n\r\n* Expand transitive closure test\r\n\r\n* Add join-with-self test that requires deterministic codegen\r\n\r\n* Address feedback\r\n\r\n* Extract join generation logic to a separate function\r\n\r\n* Eliminate assumption of usize columns\r\n\r\n* Rename datalog_compiler => hydroflow_datalog",
          "timestamp": "2022-08-24T05:53:13Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fd3867fde4302aabd747ca81564dfba6016a6395"
        },
        "date": 1661747246097,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376077,
            "range": "± 2800",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 180183025,
            "range": "± 2388880",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 102610576,
            "range": "± 6540116",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11368407,
            "range": "± 151368",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31021745,
            "range": "± 131465",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38285451,
            "range": "± 271029",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684498,
            "range": "± 4862",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2769231,
            "range": "± 66108",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2816163,
            "range": "± 29471",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "c252b0565bc86b37e5e25941ba1e9ed3c80d7863",
          "message": "Update datalog snapshot tests",
          "timestamp": "2022-08-26T17:49:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c252b0565bc86b37e5e25941ba1e9ed3c80d7863"
        },
        "date": 1661833966645,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 371250,
            "range": "± 15603",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 183668651,
            "range": "± 7340869",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 103715173,
            "range": "± 5021069",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13644279,
            "range": "± 461035",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 28891642,
            "range": "± 1463785",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47515726,
            "range": "± 2098468",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17765151,
            "range": "± 905573",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3313907,
            "range": "± 201650",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3379741,
            "range": "± 154589",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Shadaj Laddad",
            "username": "shadaj",
            "email": "shadaj@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "863fdc8fea27d3b41dd3bd94212bee515a923340",
          "message": "Generate nested joins for rules with more than two RHS relations (#184)\n\n* Generate nested joins for rules with more than two RHS relations\r\n\r\n* Add tests\r\n\r\n* Dedup code\r\n\r\n* Cleanup\r\n\r\n* Add struct capturing join plan expansion\r\n\r\n* Rename generate_join -> generate_rule\r\n\r\n* Embed target reference within JoinPlans\r\n\r\n* Rename Target => Atom\r\n\r\n* Add utility counter abstraction\r\n\r\n* Add more docs\r\n\r\n* More docs",
          "timestamp": "2022-08-30T21:54:21Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/863fdc8fea27d3b41dd3bd94212bee515a923340"
        },
        "date": 1661920427621,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376013,
            "range": "± 2540",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 158065015,
            "range": "± 1214869",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 94418236,
            "range": "± 2948503",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11473825,
            "range": "± 266486",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31733459,
            "range": "± 121626",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40434662,
            "range": "± 930994",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12965255,
            "range": "± 7567",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2459592,
            "range": "± 14539",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2500653,
            "range": "± 158776",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "09e5cfdc2c0e02d1c84251008814f1f569048b18",
          "message": "Add comments to flat_graph",
          "timestamp": "2022-08-30T19:28:04Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/09e5cfdc2c0e02d1c84251008814f1f569048b18"
        },
        "date": 1662006193448,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 331770,
            "range": "± 2748",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 158543848,
            "range": "± 4470367",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 91796767,
            "range": "± 3150579",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9501061,
            "range": "± 601866",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32008627,
            "range": "± 125079",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38985061,
            "range": "± 650056",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14686714,
            "range": "± 10223",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2456620,
            "range": "± 58989",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2502492,
            "range": "± 19314",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662093020526,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 317789,
            "range": "± 5348",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 152661931,
            "range": "± 2488353",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89677140,
            "range": "± 1074445",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10570334,
            "range": "± 148347",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32986616,
            "range": "± 242040",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50378476,
            "range": "± 713302",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18425441,
            "range": "± 187450",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2875690,
            "range": "± 10376",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2937720,
            "range": "± 33472",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662178679235,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375758,
            "range": "± 2699",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 173828257,
            "range": "± 757679",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104064909,
            "range": "± 2138570",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11983286,
            "range": "± 279952",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31279585,
            "range": "± 246324",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38975204,
            "range": "± 979087",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14687204,
            "range": "± 8742",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2723864,
            "range": "± 51841",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2790395,
            "range": "± 15883",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662265411746,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 450722,
            "range": "± 4355",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187666975,
            "range": "± 1029187",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111304383,
            "range": "± 2270632",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14317735,
            "range": "± 203377",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37887087,
            "range": "± 284506",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 46697107,
            "range": "± 1688176",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17443584,
            "range": "± 167179",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3228918,
            "range": "± 42627",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3287257,
            "range": "± 42707",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662352242067,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375887,
            "range": "± 3458",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 181476529,
            "range": "± 1793328",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108231895,
            "range": "± 1786452",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11934687,
            "range": "± 241553",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31351823,
            "range": "± 125966",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40881515,
            "range": "± 1505981",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14695569,
            "range": "± 5277",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2751436,
            "range": "± 66705",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2800467,
            "range": "± 21811",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662438759912,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375688,
            "range": "± 2534",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165483969,
            "range": "± 1517593",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100140926,
            "range": "± 1264928",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12263549,
            "range": "± 253100",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32098163,
            "range": "± 77445",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37914245,
            "range": "± 301989",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15897296,
            "range": "± 7009",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2733766,
            "range": "± 81684",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2766271,
            "range": "± 16842",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662525299269,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 394823,
            "range": "± 26969",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186855790,
            "range": "± 6024097",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 123947805,
            "range": "± 3643791",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15345017,
            "range": "± 687745",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36463206,
            "range": "± 1666246",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 59755700,
            "range": "± 4425696",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 20146020,
            "range": "± 672611",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 4260311,
            "range": "± 540163",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3798880,
            "range": "± 274491",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662611208535,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 383387,
            "range": "± 13208",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188066586,
            "range": "± 5538705",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109868780,
            "range": "± 5062822",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14581892,
            "range": "± 496474",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34405755,
            "range": "± 1133021",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52221088,
            "range": "± 3204594",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 20187259,
            "range": "± 678465",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3407757,
            "range": "± 153506",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3441773,
            "range": "± 129866",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662697260478,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 310641,
            "range": "± 6761",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 146602654,
            "range": "± 946536",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89712180,
            "range": "± 1047964",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10924790,
            "range": "± 151499",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32243595,
            "range": "± 128968",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48493732,
            "range": "± 569082",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15511549,
            "range": "± 132804",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2868770,
            "range": "± 59478",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2915487,
            "range": "± 18087",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662784018369,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 276770,
            "range": "± 9918",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 134971276,
            "range": "± 2171914",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89965238,
            "range": "± 1976560",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10515674,
            "range": "± 144501",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32781361,
            "range": "± 117972",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49919115,
            "range": "± 404899",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15866634,
            "range": "± 166153",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2896157,
            "range": "± 18551",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2915953,
            "range": "± 21188",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662870151400,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451005,
            "range": "± 2758",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 184218134,
            "range": "± 676802",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113592572,
            "range": "± 988303",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12732949,
            "range": "± 109029",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38033294,
            "range": "± 1043175",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 46261873,
            "range": "± 503737",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17637016,
            "range": "± 82897",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3406160,
            "range": "± 167622",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3350771,
            "range": "± 86247",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1662957279278,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451075,
            "range": "± 2633",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 189600133,
            "range": "± 506547",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 104493041,
            "range": "± 777450",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13773281,
            "range": "± 435185",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37518526,
            "range": "± 1295051",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50249797,
            "range": "± 2089470",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17630546,
            "range": "± 208750",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3521297,
            "range": "± 229661",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3356883,
            "range": "± 96530",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663043464386,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375947,
            "range": "± 2730",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 182525614,
            "range": "± 2341295",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113140756,
            "range": "± 2049685",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12072057,
            "range": "± 88201",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32312838,
            "range": "± 309313",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45071900,
            "range": "± 1990580",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14697693,
            "range": "± 8816",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2881785,
            "range": "± 38651",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2819410,
            "range": "± 26612",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663129740663,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 346150,
            "range": "± 26695",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 208388823,
            "range": "± 6284985",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111942461,
            "range": "± 4575861",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12054790,
            "range": "± 944313",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31298638,
            "range": "± 1715247",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48568318,
            "range": "± 2802996",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15481831,
            "range": "± 757546",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3092221,
            "range": "± 192415",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3260615,
            "range": "± 175587",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663216359381,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 357764,
            "range": "± 26037",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 186156638,
            "range": "± 6777305",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113286462,
            "range": "± 5703340",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12977743,
            "range": "± 805135",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32778035,
            "range": "± 1817592",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50516507,
            "range": "± 3858664",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15812565,
            "range": "± 1038440",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3193626,
            "range": "± 247245",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3286041,
            "range": "± 188256",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663302737012,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 334071,
            "range": "± 2911",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 161471131,
            "range": "± 4656640",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97975306,
            "range": "± 3005964",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9382979,
            "range": "± 150414",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 28163529,
            "range": "± 108558",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38959027,
            "range": "± 321475",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14684891,
            "range": "± 10599",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2615898,
            "range": "± 17063",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2632841,
            "range": "± 27897",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663388151714,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 262256,
            "range": "± 5667",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 127847468,
            "range": "± 1485205",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 87750166,
            "range": "± 877937",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11264773,
            "range": "± 128554",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32241001,
            "range": "± 138695",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50809875,
            "range": "± 336120",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18204769,
            "range": "± 17365",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2916734,
            "range": "± 8993",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2930913,
            "range": "± 29875",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663475224130,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451414,
            "range": "± 2827",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 188569825,
            "range": "± 825604",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 115181816,
            "range": "± 1197012",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15023037,
            "range": "± 579388",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 39330659,
            "range": "± 1545815",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 53346989,
            "range": "± 1027987",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18510578,
            "range": "± 1155913",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3832450,
            "range": "± 318344",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3668776,
            "range": "± 233759",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "9c9a27b42c9855ab9d725214b68d66c6c273da2b",
          "message": "Update datalog codegen snapshots",
          "timestamp": "2022-09-01T21:21:12Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/9c9a27b42c9855ab9d725214b68d66c6c273da2b"
        },
        "date": 1663562053419,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 452798,
            "range": "± 5353",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 191158251,
            "range": "± 2805156",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116805075,
            "range": "± 1525027",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15534661,
            "range": "± 718820",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37980125,
            "range": "± 118604",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50327931,
            "range": "± 1416849",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17648351,
            "range": "± 15147",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3394671,
            "range": "± 158979",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3313263,
            "range": "± 30189",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5a1777991cd72a566596b4d7b0375387c6985967",
          "message": "Book edits, fix #183 (#190)\n\n* replace refs in book: \"Core API\" -> \"Surface API\"\r\n\r\n* remove dead examples/book\r\n\r\n* split Graph Reachability into two subsections\r\n\r\n* explain vec![0] and remove refs to pull\r\n\r\n* explain vec![0]\r\n\r\n* fix link to design doc\r\n\r\n* fix bug/typo where pull should have been push\r\n\r\n* more careful discussion of the space of partitionings\r\n\r\n* restore refs to Core API, other small edits\r\n\r\n* clarify that example_4_1 illustrates multiple outputs",
          "timestamp": "2022-09-19T20:20:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/5a1777991cd72a566596b4d7b0375387c6985967"
        },
        "date": 1663648349752,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 380050,
            "range": "± 12369",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 194130623,
            "range": "± 5189650",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111355653,
            "range": "± 5764077",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13982596,
            "range": "± 495031",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35020038,
            "range": "± 1210121",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39388757,
            "range": "± 1644380",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18079102,
            "range": "± 614770",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3452353,
            "range": "± 173644",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3476653,
            "range": "± 146985",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1663734811217,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 395209,
            "range": "± 19408",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198671421,
            "range": "± 7202685",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125025985,
            "range": "± 6026844",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 15070511,
            "range": "± 797666",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35531939,
            "range": "± 1779375",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54521850,
            "range": "± 3274700",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17593691,
            "range": "± 926555",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3941822,
            "range": "± 285136",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3673043,
            "range": "± 435646",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1663820900266,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378037,
            "range": "± 2763",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 162981799,
            "range": "± 1435724",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105484815,
            "range": "± 1835605",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10636048,
            "range": "± 72400",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32015154,
            "range": "± 145679",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40944671,
            "range": "± 1035753",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14693109,
            "range": "± 7748",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2765896,
            "range": "± 11739",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2777745,
            "range": "± 19932",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1663907489684,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 370074,
            "range": "± 21007",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 174091393,
            "range": "± 6624158",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109240201,
            "range": "± 5147901",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13271760,
            "range": "± 534287",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33176197,
            "range": "± 1854740",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49275426,
            "range": "± 2524095",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17406217,
            "range": "± 1024139",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3364306,
            "range": "± 208839",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3356806,
            "range": "± 243156",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1663993636975,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 263119,
            "range": "± 2520",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 130252785,
            "range": "± 987348",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89046515,
            "range": "± 1175993",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11199064,
            "range": "± 66522",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32799005,
            "range": "± 217783",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51247344,
            "range": "± 187964",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18221655,
            "range": "± 20388",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3018951,
            "range": "± 68718",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2941463,
            "range": "± 11229",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1664079831580,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 399502,
            "range": "± 12913",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 194028167,
            "range": "± 5065886",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126461960,
            "range": "± 3943992",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14643302,
            "range": "± 699159",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37206973,
            "range": "± 978783",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 58150639,
            "range": "± 4277226",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18630467,
            "range": "± 968302",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 4182205,
            "range": "± 326271",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3779863,
            "range": "± 345022",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "8b68c643b55e9a04f373bded939b512be4ee0d7f",
          "message": "Update `recv_stream` to handle all `Stream`s instead of just `tokio::mpsc::unbounded_channel`\n\nBreaking change",
          "timestamp": "2022-09-19T21:36:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/8b68c643b55e9a04f373bded939b512be4ee0d7f"
        },
        "date": 1664166775393,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 277781,
            "range": "± 7024",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 146728077,
            "range": "± 482373",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90456024,
            "range": "± 1122855",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10441024,
            "range": "± 67706",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32326473,
            "range": "± 119955",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51235225,
            "range": "± 294095",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18224001,
            "range": "± 15113",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2956304,
            "range": "± 13626",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2971372,
            "range": "± 13611",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "3559fbfa19711447fc53dfc597ad18b9a2f81a50",
          "message": "Surface syntax fix handling of wildcard linear chains which might cause later pull-push conflicts",
          "timestamp": "2022-09-27T00:47:17Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3559fbfa19711447fc53dfc597ad18b9a2f81a50"
        },
        "date": 1664252729981,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 269110,
            "range": "± 4111",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 146380023,
            "range": "± 844749",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 87656434,
            "range": "± 1257095",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10508610,
            "range": "± 96977",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32397669,
            "range": "± 113063",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49459999,
            "range": "± 1191609",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18205103,
            "range": "± 27565",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2912461,
            "range": "± 11847",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2921271,
            "range": "± 18571",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1bf86425afacbdd23b391567330f3a48a518d6d7",
          "message": "Add `#[allow(clippy::map_identity)]` to some tests",
          "timestamp": "2022-09-27T23:21:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1bf86425afacbdd23b391567330f3a48a518d6d7"
        },
        "date": 1664339476726,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 320755,
            "range": "± 2974",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 140248092,
            "range": "± 1646199",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 93381946,
            "range": "± 1666365",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10981471,
            "range": "± 157542",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32403382,
            "range": "± 107947",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49638253,
            "range": "± 183566",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18225741,
            "range": "± 11872",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2822184,
            "range": "± 3061",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2897023,
            "range": "± 3781",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "1bf86425afacbdd23b391567330f3a48a518d6d7",
          "message": "Add `#[allow(clippy::map_identity)]` to some tests",
          "timestamp": "2022-09-27T23:21:50Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/1bf86425afacbdd23b391567330f3a48a518d6d7"
        },
        "date": 1664425849980,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 377738,
            "range": "± 2634",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 172803656,
            "range": "± 3104730",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108345785,
            "range": "± 2258000",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10833329,
            "range": "± 53074",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31631955,
            "range": "± 90218",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 40587096,
            "range": "± 908669",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691898,
            "range": "± 7504",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2591338,
            "range": "± 10170",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2605603,
            "range": "± 3162",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dfc22e84e640baa93b76cdf6cf7702684e55ea0a",
          "message": "Add assertiosn to surface tcp test",
          "timestamp": "2022-09-29T21:03:25Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dfc22e84e640baa93b76cdf6cf7702684e55ea0a"
        },
        "date": 1664512556175,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 383041,
            "range": "± 19620",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 160280390,
            "range": "± 4586005",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 107503012,
            "range": "± 3972810",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14852427,
            "range": "± 691763",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34967704,
            "range": "± 1461296",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52846867,
            "range": "± 3417782",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18247480,
            "range": "± 940604",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3293201,
            "range": "± 195240",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3310064,
            "range": "± 213793",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dfc22e84e640baa93b76cdf6cf7702684e55ea0a",
          "message": "Add assertiosn to surface tcp test",
          "timestamp": "2022-09-29T21:03:25Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dfc22e84e640baa93b76cdf6cf7702684e55ea0a"
        },
        "date": 1664598799965,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376019,
            "range": "± 3879",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 154949852,
            "range": "± 518937",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97167056,
            "range": "± 2342578",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10171906,
            "range": "± 268863",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31703330,
            "range": "± 71733",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39420119,
            "range": "± 620769",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14686276,
            "range": "± 7671",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2549523,
            "range": "± 12174",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2614756,
            "range": "± 2548",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dfc22e84e640baa93b76cdf6cf7702684e55ea0a",
          "message": "Add assertiosn to surface tcp test",
          "timestamp": "2022-09-29T21:03:25Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dfc22e84e640baa93b76cdf6cf7702684e55ea0a"
        },
        "date": 1664684863844,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 332059,
            "range": "± 17799",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 167509283,
            "range": "± 716444",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 100173964,
            "range": "± 1324446",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 8968927,
            "range": "± 115577",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 28200006,
            "range": "± 80780",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 38546095,
            "range": "± 1224850",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 12966382,
            "range": "± 18938",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2264927,
            "range": "± 4490",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2851564,
            "range": "± 7674",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "dfc22e84e640baa93b76cdf6cf7702684e55ea0a",
          "message": "Add assertiosn to surface tcp test",
          "timestamp": "2022-09-29T21:03:25Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dfc22e84e640baa93b76cdf6cf7702684e55ea0a"
        },
        "date": 1664769926945,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 390571,
            "range": "± 5395",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 182227515,
            "range": "± 4411079",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 114131192,
            "range": "± 2510184",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13756131,
            "range": "± 463399",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35157937,
            "range": "± 536978",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 54735645,
            "range": "± 761916",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18199648,
            "range": "± 369480",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3253282,
            "range": "± 87046",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3424593,
            "range": "± 71140",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1664856627203,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 435087,
            "range": "± 9550",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 180849330,
            "range": "± 2592413",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111204968,
            "range": "± 1909897",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11738852,
            "range": "± 263621",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36353978,
            "range": "± 733906",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45290357,
            "range": "± 1373573",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17128796,
            "range": "± 291042",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2985986,
            "range": "± 49557",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3093416,
            "range": "± 67963",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1664942657157,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 318798,
            "range": "± 4185",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 140572159,
            "range": "± 1208389",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89366924,
            "range": "± 1537825",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10201227,
            "range": "± 188549",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32080465,
            "range": "± 105671",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50872238,
            "range": "± 188745",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18225057,
            "range": "± 455322",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2766362,
            "range": "± 6797",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2815545,
            "range": "± 2529",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665029083773,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 438080,
            "range": "± 10159",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 178349843,
            "range": "± 1415751",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108316796,
            "range": "± 1690446",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11441389,
            "range": "± 273631",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35702654,
            "range": "± 541595",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 44649390,
            "range": "± 1274017",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16962496,
            "range": "± 356698",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2953239,
            "range": "± 67299",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3033241,
            "range": "± 60479",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665115747744,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375636,
            "range": "± 2708",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 158271929,
            "range": "± 1978477",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96826792,
            "range": "± 1071533",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9940422,
            "range": "± 32812",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31292642,
            "range": "± 93222",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39060352,
            "range": "± 259849",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14690950,
            "range": "± 7826",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2622329,
            "range": "± 10658",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2610517,
            "range": "± 6440",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665201811191,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378103,
            "range": "± 3574",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 153597162,
            "range": "± 1025248",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99705101,
            "range": "± 671158",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9406368,
            "range": "± 96113",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30584915,
            "range": "± 32544",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39541428,
            "range": "± 336122",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681779,
            "range": "± 6505",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2542614,
            "range": "± 11593",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2582189,
            "range": "± 2620",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665289249183,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 440785,
            "range": "± 9455",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 180987091,
            "range": "± 2630248",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 106972273,
            "range": "± 3423521",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10992471,
            "range": "± 214387",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36726657,
            "range": "± 239570",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47789839,
            "range": "± 680650",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17631144,
            "range": "± 102678",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3017123,
            "range": "± 76483",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3093957,
            "range": "± 9222",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665376215866,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 469721,
            "range": "± 3319",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 187182262,
            "range": "± 1821514",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 111079123,
            "range": "± 948485",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12345798,
            "range": "± 212886",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38046813,
            "range": "± 123300",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48191855,
            "range": "± 386873",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18363096,
            "range": "± 22408",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3177040,
            "range": "± 13740",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3210395,
            "range": "± 13812",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "fa20c12f10025133a071e46af67f858366e6e0da",
          "message": "Fix #201 `run_async()`, wait when ALL strata have no work\n\nInstead of waiting when ANY stratum has no work",
          "timestamp": "2022-10-03T18:58:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/fa20c12f10025133a071e46af67f858366e6e0da"
        },
        "date": 1665462513218,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 409408,
            "range": "± 17742",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 203604236,
            "range": "± 6475013",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 126976766,
            "range": "± 6720471",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13228614,
            "range": "± 804057",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38137869,
            "range": "± 1740199",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 56676472,
            "range": "± 4266936",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18978334,
            "range": "± 791338",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3579638,
            "range": "± 277988",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3557475,
            "range": "± 258676",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6d9616e8740a98f16fbff84fa5b6e8295a1d9a15",
          "message": "Update datalog snapshots",
          "timestamp": "2022-10-04T16:37:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6d9616e8740a98f16fbff84fa5b6e8295a1d9a15"
        },
        "date": 1665548556687,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 469659,
            "range": "± 2755",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 172752084,
            "range": "± 774697",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109667991,
            "range": "± 1344281",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12032101,
            "range": "± 323020",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 38598221,
            "range": "± 103019",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 47164537,
            "range": "± 310364",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18357588,
            "range": "± 13499",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3493038,
            "range": "± 24036",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3212578,
            "range": "± 2417",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6d9616e8740a98f16fbff84fa5b6e8295a1d9a15",
          "message": "Update datalog snapshots",
          "timestamp": "2022-10-04T16:37:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6d9616e8740a98f16fbff84fa5b6e8295a1d9a15"
        },
        "date": 1665635335686,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375728,
            "range": "± 2455",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 154277281,
            "range": "± 1416409",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98890359,
            "range": "± 2016307",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9488087,
            "range": "± 264823",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31004091,
            "range": "± 101975",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37928386,
            "range": "± 808242",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679569,
            "range": "± 15634",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2555072,
            "range": "± 23808",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2700721,
            "range": "± 4079",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "6d9616e8740a98f16fbff84fa5b6e8295a1d9a15",
          "message": "Update datalog snapshots",
          "timestamp": "2022-10-04T16:37:37Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/6d9616e8740a98f16fbff84fa5b6e8295a1d9a15"
        },
        "date": 1665722075556,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 348842,
            "range": "± 19484",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 154775268,
            "range": "± 5447299",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 99087245,
            "range": "± 4639556",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12319973,
            "range": "± 715217",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31911765,
            "range": "± 1213482",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 44847285,
            "range": "± 3540036",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16032526,
            "range": "± 478785",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2898660,
            "range": "± 111694",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2952839,
            "range": "± 78846",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c526f9a70de0d9a5d15655ad99412f3b425b4cab",
          "message": "Add cross join surface syntax operator, update tests, fix #200 (#211)",
          "timestamp": "2022-10-14T22:10:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c526f9a70de0d9a5d15655ad99412f3b425b4cab"
        },
        "date": 1665808035806,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 299192,
            "range": "± 16590",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 132392204,
            "range": "± 1002140",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 88873985,
            "range": "± 1880064",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10071974,
            "range": "± 195026",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32530889,
            "range": "± 106833",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49959219,
            "range": "± 173703",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18209841,
            "range": "± 22719",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2780476,
            "range": "± 8020",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2810206,
            "range": "± 2892",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c526f9a70de0d9a5d15655ad99412f3b425b4cab",
          "message": "Add cross join surface syntax operator, update tests, fix #200 (#211)",
          "timestamp": "2022-10-14T22:10:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c526f9a70de0d9a5d15655ad99412f3b425b4cab"
        },
        "date": 1665894428215,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 376054,
            "range": "± 4691",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 155030080,
            "range": "± 1284655",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98285782,
            "range": "± 1015696",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10488430,
            "range": "± 268576",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31064650,
            "range": "± 48510",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39454768,
            "range": "± 405958",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14699150,
            "range": "± 6689",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2560434,
            "range": "± 7938",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2621716,
            "range": "± 2553",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c526f9a70de0d9a5d15655ad99412f3b425b4cab",
          "message": "Add cross join surface syntax operator, update tests, fix #200 (#211)",
          "timestamp": "2022-10-14T22:10:40Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/c526f9a70de0d9a5d15655ad99412f3b425b4cab"
        },
        "date": 1665981324090,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 380321,
            "range": "± 27231",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 205639640,
            "range": "± 7126341",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 122507999,
            "range": "± 6719479",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12527280,
            "range": "± 965468",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35988365,
            "range": "± 2107988",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52351424,
            "range": "± 4235069",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17806164,
            "range": "± 1058645",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3283853,
            "range": "± 231406",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3297326,
            "range": "± 240711",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "7797c6c4aff07f780069bb9af2b12b8999b33725",
          "message": "Implement `inspect()` surface syntax operator, fix #208",
          "timestamp": "2022-10-13T19:32:55Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/7797c6c4aff07f780069bb9af2b12b8999b33725"
        },
        "date": 1666067589017,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451096,
            "range": "± 2878",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 172380115,
            "range": "± 1778262",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 110111694,
            "range": "± 2160682",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11537694,
            "range": "± 124091",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36736878,
            "range": "± 118143",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45628395,
            "range": "± 808161",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17561086,
            "range": "± 100204",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3031667,
            "range": "± 23722",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3059470,
            "range": "± 12164",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f802b9536cf9d07846e2ace54b09786c919aea11",
          "message": "add flatten op to surface syntax (#213)\n\n* add flatten op to surface syntax\r\n\r\n* handle push case in flatten and test it",
          "timestamp": "2022-10-18T23:23:18Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f802b9536cf9d07846e2ace54b09786c919aea11"
        },
        "date": 1666154068890,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 379351,
            "range": "± 16418",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 178923260,
            "range": "± 16211462",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 116876237,
            "range": "± 5618411",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13572156,
            "range": "± 986330",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34858953,
            "range": "± 1473384",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 61494159,
            "range": "± 4886228",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18406969,
            "range": "± 636374",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3476419,
            "range": "± 167837",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3494137,
            "range": "± 195901",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ea99408665e4157ba1129a8401ad3eeb850eed84",
          "message": "use sequential numbers to index tee output",
          "timestamp": "2022-10-20T02:29:27Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/ea99408665e4157ba1129a8401ad3eeb850eed84"
        },
        "date": 1666239552883,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 285750,
            "range": "± 21172",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 131709015,
            "range": "± 1358916",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90685640,
            "range": "± 748330",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9208341,
            "range": "± 38944",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32502397,
            "range": "± 146099",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49342970,
            "range": "± 151021",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18204804,
            "range": "± 19111",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2817266,
            "range": "± 5440",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2827186,
            "range": "± 3214",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "af550ca7c787d705b54532540f078b9d3e5d999b",
          "message": "Remove Surface API from book",
          "timestamp": "2022-10-26T21:47:19Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/af550ca7c787d705b54532540f078b9d3e5d999b"
        },
        "date": 1667276729176,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 321169,
            "range": "± 3332",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 133053600,
            "range": "± 878626",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 86770703,
            "range": "± 1602429",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9254678,
            "range": "± 254712",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32070258,
            "range": "± 109012",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49757748,
            "range": "± 538537",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18215090,
            "range": "± 20181",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2764204,
            "range": "± 7994",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2802696,
            "range": "± 4062",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "879e977205f055e9712c2887a275dcdbee49f540",
          "message": "Implement named ports in operators\n\n- `pos`, `neg` required for `-> difference()`\n- `0`, `1` required for `-> join()`",
          "timestamp": "2022-10-28T02:20:00Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/879e977205f055e9712c2887a275dcdbee49f540"
        },
        "date": 1667362656951,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 342286,
            "range": "± 14883",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 166687207,
            "range": "± 5898114",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101686354,
            "range": "± 3087685",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11676899,
            "range": "± 454935",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31258256,
            "range": "± 1035509",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 43185664,
            "range": "± 1497305",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16357289,
            "range": "± 578684",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2895444,
            "range": "± 106364",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2963458,
            "range": "± 132812",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "adbf8e124073eac14e667c40783c8b931ec04ccf",
          "message": "Add cargo audit to gh-actions",
          "timestamp": "2022-11-02T20:23:49Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/adbf8e124073eac14e667c40783c8b931ec04ccf"
        },
        "date": 1667448172954,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375773,
            "range": "± 2983",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 147429814,
            "range": "± 2392850",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 98834732,
            "range": "± 2402341",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9255379,
            "range": "± 312967",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30774687,
            "range": "± 55475",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 39691072,
            "range": "± 2677852",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691381,
            "range": "± 9959",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2536843,
            "range": "± 7824",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2590414,
            "range": "± 4547",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b8394d8da3479be55a19fe5743285d8480f78c61",
          "message": "Add testing of surface syntax errors (and warnings) (#230)",
          "timestamp": "2022-11-04T00:53:01Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b8394d8da3479be55a19fe5743285d8480f78c61"
        },
        "date": 1667534840329,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 427516,
            "range": "± 38114",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 203542303,
            "range": "± 10758868",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 124544662,
            "range": "± 8814875",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 14366992,
            "range": "± 1641490",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 41930798,
            "range": "± 3406703",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 60080710,
            "range": "± 5600568",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18582048,
            "range": "± 917807",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3441174,
            "range": "± 216343",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3514851,
            "range": "± 289315",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f",
          "message": "Move type list code into `type_list` subpackage (#231)",
          "timestamp": "2022-11-04T23:04:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f"
        },
        "date": 1667620807419,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375894,
            "range": "± 4041",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 154103611,
            "range": "± 529327",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96379061,
            "range": "± 2142116",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9640825,
            "range": "± 623360",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30331874,
            "range": "± 51755",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37902140,
            "range": "± 288666",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14681283,
            "range": "± 8583",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2535703,
            "range": "± 6684",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2579458,
            "range": "± 3295",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f",
          "message": "Move type list code into `type_list` subpackage (#231)",
          "timestamp": "2022-11-04T23:04:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f"
        },
        "date": 1667707305809,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 352053,
            "range": "± 13543",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 163520166,
            "range": "± 6762578",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 101194758,
            "range": "± 4296693",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12746109,
            "range": "± 589905",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32278849,
            "range": "± 1443570",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 45553945,
            "range": "± 2332423",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16757940,
            "range": "± 776261",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2995580,
            "range": "± 160205",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3145795,
            "range": "± 196176",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f",
          "message": "Move type list code into `type_list` subpackage (#231)",
          "timestamp": "2022-11-04T23:04:53Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3b7c9981a268b0bc6023e4c9d73c9dbceee02a4f"
        },
        "date": 1667793990689,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 410616,
            "range": "± 19038",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 201273550,
            "range": "± 6642748",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 128571702,
            "range": "± 6211172",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13491100,
            "range": "± 503455",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 36577339,
            "range": "± 2204349",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 57631741,
            "range": "± 3568382",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18196955,
            "range": "± 1127547",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3419206,
            "range": "± 203983",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3553893,
            "range": "± 226861",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tyler Hou",
            "username": "tylerhou",
            "email": "tyler.hou.cs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b2357466115dd2fe6257da01af855840f1ff33c9",
          "message": "Add surface graph snapshot tests for datalog. (#223)",
          "timestamp": "2022-11-07T06:31:33Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b2357466115dd2fe6257da01af855840f1ff33c9"
        },
        "date": 1667880306447,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 434373,
            "range": "± 11201",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 168347762,
            "range": "± 2194883",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109078046,
            "range": "± 2797301",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10607149,
            "range": "± 350480",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35023827,
            "range": "± 1082213",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41600571,
            "range": "± 1747158",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16256140,
            "range": "± 517531",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2961332,
            "range": "± 108913",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2922052,
            "range": "± 99759",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tyler Hou",
            "username": "tylerhou",
            "email": "tyler.hou.cs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b2357466115dd2fe6257da01af855840f1ff33c9",
          "message": "Add surface graph snapshot tests for datalog. (#223)",
          "timestamp": "2022-11-07T06:31:33Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b2357466115dd2fe6257da01af855840f1ff33c9"
        },
        "date": 1667966715075,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 318611,
            "range": "± 18744",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 170287330,
            "range": "± 7939574",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 97750729,
            "range": "± 4963346",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 11792399,
            "range": "± 1109802",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30072981,
            "range": "± 1893522",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 42491114,
            "range": "± 2797627",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14696511,
            "range": "± 927564",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2646135,
            "range": "± 224705",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3034330,
            "range": "± 178003",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tyler Hou",
            "username": "tylerhou",
            "email": "tyler.hou.cs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b2357466115dd2fe6257da01af855840f1ff33c9",
          "message": "Add surface graph snapshot tests for datalog. (#223)",
          "timestamp": "2022-11-07T06:31:33Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b2357466115dd2fe6257da01af855840f1ff33c9"
        },
        "date": 1668052969398,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 273576,
            "range": "± 9193",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 135794390,
            "range": "± 2236043",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90262299,
            "range": "± 2393239",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9162171,
            "range": "± 30528",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31990260,
            "range": "± 140970",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 50156291,
            "range": "± 997662",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18220361,
            "range": "± 9794",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2752450,
            "range": "± 6684",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2792452,
            "range": "± 3882",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "016b97112e3f417a81b333f988367d9689d0ce55",
          "message": "Add split and switch pusherators (#233)",
          "timestamp": "2022-11-10T22:06:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/016b97112e3f417a81b333f988367d9689d0ce55"
        },
        "date": 1668139430980,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375766,
            "range": "± 2767",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 155490571,
            "range": "± 1209236",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 96455669,
            "range": "± 1102681",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9571023,
            "range": "± 54774",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 30703658,
            "range": "± 94934",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37193913,
            "range": "± 630514",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14688838,
            "range": "± 5474",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2528363,
            "range": "± 7960",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2606594,
            "range": "± 3244",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "016b97112e3f417a81b333f988367d9689d0ce55",
          "message": "Add split and switch pusherators (#233)",
          "timestamp": "2022-11-10T22:06:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/016b97112e3f417a81b333f988367d9689d0ce55"
        },
        "date": 1668225727493,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 318408,
            "range": "± 9345",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 140453588,
            "range": "± 1147564",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 94549731,
            "range": "± 2004236",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9149694,
            "range": "± 138312",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32624859,
            "range": "± 108587",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49992311,
            "range": "± 102436",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18226339,
            "range": "± 13350",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2781098,
            "range": "± 11809",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2841103,
            "range": "± 2532",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "016b97112e3f417a81b333f988367d9689d0ce55",
          "message": "Add split and switch pusherators (#233)",
          "timestamp": "2022-11-10T22:06:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/016b97112e3f417a81b333f988367d9689d0ce55"
        },
        "date": 1668312260066,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 379601,
            "range": "± 13972",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 175775076,
            "range": "± 6202391",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 109451903,
            "range": "± 3789165",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12812843,
            "range": "± 563331",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 33823374,
            "range": "± 1053961",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49329540,
            "range": "± 2534237",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18010091,
            "range": "± 618564",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3172790,
            "range": "± 111162",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3208131,
            "range": "± 89795",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "016b97112e3f417a81b333f988367d9689d0ce55",
          "message": "Add split and switch pusherators (#233)",
          "timestamp": "2022-11-10T22:06:52Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/016b97112e3f417a81b333f988367d9689d0ce55"
        },
        "date": 1668398608816,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 320969,
            "range": "± 6596",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 131495396,
            "range": "± 1613749",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 91065613,
            "range": "± 1542646",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9461354,
            "range": "± 322862",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32202412,
            "range": "± 96154",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49581430,
            "range": "± 37855",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18215276,
            "range": "± 15477",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2802323,
            "range": "± 8178",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2861032,
            "range": "± 3013",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b90c5460bc66d0386725ba8dae313f27a8dceca1",
          "message": "deadlock detector example (#234)\n\n* first draft of deadlock detector\r\n\r\n* niceties for clippy, docs\r\n\r\n* address feedback",
          "timestamp": "2022-11-15T02:00:24Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/b90c5460bc66d0386725ba8dae313f27a8dceca1"
        },
        "date": 1668484750305,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 277192,
            "range": "± 10322",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 131523102,
            "range": "± 1140255",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 89193045,
            "range": "± 2160663",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9346417,
            "range": "± 51955",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32373165,
            "range": "± 114423",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49551207,
            "range": "± 123776",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18217871,
            "range": "± 14025",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2746127,
            "range": "± 5466",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2775390,
            "range": "± 2326",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e3e8db208606bd354426332ca128a894f0e9f76e",
          "message": "add unique operator to remove duplicates (#236)",
          "timestamp": "2022-11-16T00:31:57Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/e3e8db208606bd354426332ca128a894f0e9f76e"
        },
        "date": 1668571259923,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 451185,
            "range": "± 11219",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 175196349,
            "range": "± 626877",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 113176293,
            "range": "± 858808",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12385250,
            "range": "± 80024",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 37332301,
            "range": "± 158981",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 49850955,
            "range": "± 1216686",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17639866,
            "range": "± 10795",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3132522,
            "range": "± 21228",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3095714,
            "range": "± 19846",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2230af8282ed9ddeb2a2e502a6e4737b9d0a2b0d",
          "message": "Add a driver for the chat example (#237)\n\nClients can take messages in on stdin from any process, but in order to drive them autonomously we\r\nneed a reliable and continuous source of input that's less spammy than commands like 'yes'. This\r\ndiff adds a chat driver that spits out a random phrase every few seconds to serve as this driver.",
          "timestamp": "2022-11-16T18:36:30Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/2230af8282ed9ddeb2a2e502a6e4737b9d0a2b0d"
        },
        "date": 1668657391537,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375620,
            "range": "± 2215",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 150187384,
            "range": "± 723697",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 92220309,
            "range": "± 442265",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9792168,
            "range": "± 48017",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31866737,
            "range": "± 24469",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37154475,
            "range": "± 56671",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14682001,
            "range": "± 10766",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2526826,
            "range": "± 12430",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2587136,
            "range": "± 3188",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tyler Hou",
            "username": "tylerhou",
            "email": "tylerhou@google.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3b79280d900458b38be0cbc48c669465447f4873",
          "message": "Extract parts of `expand_join_plan` into new functions. (#232)",
          "timestamp": "2022-11-17T20:53:20Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/3b79280d900458b38be0cbc48c669465447f4873"
        },
        "date": 1668743911242,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 383528,
            "range": "± 16099",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 165957402,
            "range": "± 5403110",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 108846257,
            "range": "± 4723673",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 13403850,
            "range": "± 710123",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 34396588,
            "range": "± 1460460",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 51101141,
            "range": "± 2387712",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18020323,
            "range": "± 738628",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3229558,
            "range": "± 143849",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3347404,
            "range": "± 129039",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a5de404cd06c10137f7584d152269327c698a65d",
          "message": "Add `hydroflow_macr/build.rs` to autogen operator book docs\n\n- Generated operator docs put into into `book/surface_ops.gen.md`",
          "timestamp": "2022-11-18T18:44:09Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a5de404cd06c10137f7584d152269327c698a65d"
        },
        "date": 1668830103552,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 261714,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 135825484,
            "range": "± 2153545",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 90212526,
            "range": "± 952418",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10858021,
            "range": "± 160049",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 32705707,
            "range": "± 111734",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 48762060,
            "range": "± 217193",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18227015,
            "range": "± 12925",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2796408,
            "range": "± 7068",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2894377,
            "range": "± 1529",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a5de404cd06c10137f7584d152269327c698a65d",
          "message": "Add `hydroflow_macr/build.rs` to autogen operator book docs\n\n- Generated operator docs put into into `book/surface_ops.gen.md`",
          "timestamp": "2022-11-18T18:44:09Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a5de404cd06c10137f7584d152269327c698a65d"
        },
        "date": 1668916716171,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 378373,
            "range": "± 2847",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 143441914,
            "range": "± 957261",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 93979270,
            "range": "± 723049",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 10583795,
            "range": "± 165213",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31023941,
            "range": "± 54751",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 37635860,
            "range": "± 195486",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14679907,
            "range": "± 11051",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2570217,
            "range": "± 9340",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2644416,
            "range": "± 2500",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "a5de404cd06c10137f7584d152269327c698a65d",
          "message": "Add `hydroflow_macr/build.rs` to autogen operator book docs\n\n- Generated operator docs put into into `book/surface_ops.gen.md`",
          "timestamp": "2022-11-18T18:44:09Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/a5de404cd06c10137f7584d152269327c698a65d"
        },
        "date": 1669003227128,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 375900,
            "range": "± 5140",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 167275012,
            "range": "± 2331654",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 105408988,
            "range": "± 1899354",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 9763550,
            "range": "± 210367",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 31462954,
            "range": "± 76863",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 41816388,
            "range": "± 1549072",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 14691651,
            "range": "± 7967",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 2554121,
            "range": "± 20714",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 2591386,
            "range": "± 3311",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joe Hellerstein",
            "username": "jhellerstein",
            "email": "jmh@berkeley.edu"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "26e4cfe7354230907d9dc32737d3ceb877f9195c",
          "message": "book docs for ops (#245)\n\n* more op docs\r\n\r\n* more op docs\r\n\r\n* clean up example code to pass tests\r\n\r\n* Format to address PR comments\r\n\r\nCo-authored-by: Mingwei Samuel <mingwei.samuel@gmail.com>",
          "timestamp": "2022-11-21T19:45:51Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/26e4cfe7354230907d9dc32737d3ceb877f9195c"
        },
        "date": 1669089452475,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 401947,
            "range": "± 16137",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 198639237,
            "range": "± 6166013",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 125539591,
            "range": "± 5470375",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 12646777,
            "range": "± 571108",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 35968099,
            "range": "± 1954100",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 52634093,
            "range": "± 3265323",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 17803549,
            "range": "± 895286",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 3368503,
            "range": "± 158714",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 3373647,
            "range": "± 146009",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "committer": {
            "name": "Mingwei Samuel",
            "username": "MingweiSamuel",
            "email": "mingwei.samuel@gmail.com"
          },
          "id": "f5d141d84bcc99b8bf651344d3f6ec50134dc1ac",
          "message": "Fix for new clippy lifetime lint",
          "timestamp": "2022-11-23T02:26:23Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/f5d141d84bcc99b8bf651344d3f6ec50134dc1ac"
        },
        "date": 1669175488694,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 602224,
            "range": "± 8992",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 306126207,
            "range": "± 1716023",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 196463643,
            "range": "± 649963",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 18770161,
            "range": "± 84524",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 56789771,
            "range": "± 110034",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 98714741,
            "range": "± 252394",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 16522646,
            "range": "± 886108",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 5414573,
            "range": "± 39981",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 5127871,
            "range": "± 28514",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a",
          "message": "Allow the chat example to bind and connect to DNS names (#249)\n\n* Allow the chat example to bind and connect to DNS names\r\n\r\nIn many circumstances when testing chat in a distributed setting, we won't know the right IP address\r\nto either bind or connect to in advance. This diff allows the chat client and server to bind to\r\nDNS names, which (coupled with the distributed runtime's service discovery mechanism) should make it\r\neasier for clients and servers to find each other.\r\n\r\n* Fix formatting\r\n\r\n* Responding to code review feedback\r\n\r\n* Whoops, forgot to save\r\n\r\n* Remove unnecessary return statement",
          "timestamp": "2022-11-23T22:16:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a"
        },
        "date": 1669261778737,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 1003685,
            "range": "± 11479",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 319340265,
            "range": "± 2155621",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 211507353,
            "range": "± 411437",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 19665156,
            "range": "± 85268",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 60452499,
            "range": "± 105614",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 101216864,
            "range": "± 613789",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15819546,
            "range": "± 3139",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 5019134,
            "range": "± 7851",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 4653360,
            "range": "± 5443",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a",
          "message": "Allow the chat example to bind and connect to DNS names (#249)\n\n* Allow the chat example to bind and connect to DNS names\r\n\r\nIn many circumstances when testing chat in a distributed setting, we won't know the right IP address\r\nto either bind or connect to in advance. This diff allows the chat client and server to bind to\r\nDNS names, which (coupled with the distributed runtime's service discovery mechanism) should make it\r\neasier for clients and servers to find each other.\r\n\r\n* Fix formatting\r\n\r\n* Responding to code review feedback\r\n\r\n* Whoops, forgot to save\r\n\r\n* Remove unnecessary return statement",
          "timestamp": "2022-11-23T22:16:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a"
        },
        "date": 1669348462066,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 668730,
            "range": "± 37624",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 385119436,
            "range": "± 9481532",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 267118979,
            "range": "± 8943321",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 22073929,
            "range": "± 924541",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 67942495,
            "range": "± 3014322",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 116510889,
            "range": "± 5942350",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 19055321,
            "range": "± 840109",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 6187413,
            "range": "± 346284",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 5914024,
            "range": "± 305719",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a",
          "message": "Allow the chat example to bind and connect to DNS names (#249)\n\n* Allow the chat example to bind and connect to DNS names\r\n\r\nIn many circumstances when testing chat in a distributed setting, we won't know the right IP address\r\nto either bind or connect to in advance. This diff allows the chat client and server to bind to\r\nDNS names, which (coupled with the distributed runtime's service discovery mechanism) should make it\r\neasier for clients and servers to find each other.\r\n\r\n* Fix formatting\r\n\r\n* Responding to code review feedback\r\n\r\n* Whoops, forgot to save\r\n\r\n* Remove unnecessary return statement",
          "timestamp": "2022-11-23T22:16:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a"
        },
        "date": 1669434516920,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 602535,
            "range": "± 11973",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 316820670,
            "range": "± 884332",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 202370130,
            "range": "± 1109440",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 20144247,
            "range": "± 60158",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 56712670,
            "range": "± 179835",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 106892785,
            "range": "± 351435",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 15808842,
            "range": "± 184379",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 5923609,
            "range": "± 16397",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 5597118,
            "range": "± 15371",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a",
          "message": "Allow the chat example to bind and connect to DNS names (#249)\n\n* Allow the chat example to bind and connect to DNS names\r\n\r\nIn many circumstances when testing chat in a distributed setting, we won't know the right IP address\r\nto either bind or connect to in advance. This diff allows the chat client and server to bind to\r\nDNS names, which (coupled with the distributed runtime's service discovery mechanism) should make it\r\neasier for clients and servers to find each other.\r\n\r\n* Fix formatting\r\n\r\n* Responding to code review feedback\r\n\r\n* Whoops, forgot to save\r\n\r\n* Remove unnecessary return statement",
          "timestamp": "2022-11-23T22:16:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a"
        },
        "date": 1669521282297,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 794419,
            "range": "± 48490",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 369453567,
            "range": "± 9614175",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 237094449,
            "range": "± 5003617",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 28382555,
            "range": "± 471576",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 66278042,
            "range": "± 1087722",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 120988894,
            "range": "± 5067831",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18485829,
            "range": "± 714920",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 5968272,
            "range": "± 241513",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 5386762,
            "range": "± 101588",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Alex Rasmussen",
            "username": "alexras",
            "email": "535829+alexras@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a",
          "message": "Allow the chat example to bind and connect to DNS names (#249)\n\n* Allow the chat example to bind and connect to DNS names\r\n\r\nIn many circumstances when testing chat in a distributed setting, we won't know the right IP address\r\nto either bind or connect to in advance. This diff allows the chat client and server to bind to\r\nDNS names, which (coupled with the distributed runtime's service discovery mechanism) should make it\r\neasier for clients and servers to find each other.\r\n\r\n* Fix formatting\r\n\r\n* Responding to code review feedback\r\n\r\n* Whoops, forgot to save\r\n\r\n* Remove unnecessary return statement",
          "timestamp": "2022-11-23T22:16:10Z",
          "url": "https://github.com/hydro-project/hydroflow/commit/dd3d9e16f9f4fa4ec1426430dd28e4af2a6f445a"
        },
        "date": 1669607507758,
        "tool": "cargo",
        "benches": [
          {
            "name": "arithmetic/hydroflow/compiled",
            "value": 730424,
            "range": "± 30414",
            "unit": "ns/iter"
          },
          {
            "name": "fan_in/hydroflow",
            "value": 363337644,
            "range": "± 8405232",
            "unit": "ns/iter"
          },
          {
            "name": "fan_out/hydroflow/scheduled",
            "value": 217302712,
            "range": "± 5553707",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow",
            "value": 21079289,
            "range": "± 826999",
            "unit": "ns/iter"
          },
          {
            "name": "fork_join/hydroflow_builder",
            "value": 67337742,
            "range": "± 3054012",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow",
            "value": 114051665,
            "range": "± 2880051",
            "unit": "ns/iter"
          },
          {
            "name": "identity/hydroflow/compiled",
            "value": 18176138,
            "range": "± 477122",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow/scheduled",
            "value": 5586557,
            "range": "± 158803",
            "unit": "ns/iter"
          },
          {
            "name": "reachability/hydroflow",
            "value": 5252748,
            "range": "± 114158",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}