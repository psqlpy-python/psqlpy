---
title: Benchmarks
---
You can find our benchmark setup [here](https://github.com/psqlpy-python/psqlpy/tree/main/psqlpy-stress).
These benchmarks aren't isolated from real-production situation.
We've performed benchmarks with:
- AioHTTP as a web framework
- External and Local Database

We checked the maximum possible RPS for the same SQL query in the same configuration.

There are two types of benchmarks:
1) With local database
2) With external database

In a real production system database doesn't usually locate on the same server where application runs, so if you application and database are located on the different servers, we recommend you to check `External Database`.

::: important
For local benchmarks we use 5 connections in a connection pool and 10 processes make requests to the application, while for external database checks we use 40 connections and 100 processes make requests to the application.
The main reason is external database is located on a very powerful server and can serve more requests.
:::

## Key findings
If your application and database are located on the same server, there is no significant difference between `AsyncPG`, `PsycoPG` and `PSQLPy` but still you will have performance improve by approximately 10%.
However, if you have application and database located on different machines, you can get significant (up to 3 times) boost in performance.

## Local Database
::: tabs
@tab Simple Connection Select
::: chart Simple Connection Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [485.28],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [419.11],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [410.7],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
@tab Hard Connection Select
::: chart Hard Connection Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [207.57],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [194.55],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [209.72],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
@tab Combined Connection Query
::: chart Combined Connection Query

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [411.83],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [194.55],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [209.72],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
@tab Simple Transaction Select
::: chart Simple Transaction Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [451.97],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [414.33],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [401.92],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
@tab Hard Transaction Select
::: chart Hard Transaction Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [196.64],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [168.18],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [168.64],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
@tab Combined Transaction Query
::: chart Combined Transaction Query

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [369.87],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [331.44],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [324.11],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
:::

## External Database
::: tabs
@tab Simple Connection Select
::: chart Simple Connection Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [927.39],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [433.51],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [303.36],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```

@tab Hard Connection Select
::: chart Hard Connection Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [886.81],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [428.65],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [298.47],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```

@tab Combined Connection Query
::: chart Combined Connection Query

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [441.20],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [282.46],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [217.54],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```

@tab Simple Transaction Select
::: chart Simple Transaction Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [331.44],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [223.51],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [271.50],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```

@tab Hard Transaction Select
::: chart Hard Transaction Select

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [314.35],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [220.67],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [271.15],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```

@tab Combined Transaction Query
::: chart Combined Transaction Query

```json
{
  "type": "bar",
  "data": {
    "labels": ["Requests Per Second"],
    "datasets": [
      {
        "label": "PSQLPy",
        "data": [247.99],
        "backgroundColor": [
          "rgba(255, 99, 132, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 99, 132, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "AsyncPG",
        "data": [178.41],
        "backgroundColor": [
          "rgba(54, 162, 235, 0.2)"
        ],
        "borderColor": [
          "rgba(54, 162, 235, 1)"
        ],
        "borderWidth": 1
      },
      {
        "label": "PsycoPG 3",
        "data": [202.19],
        "backgroundColor": [
          "rgba(255, 206, 86, 0.2)"
        ],
        "borderColor": [
          "rgba(255, 206, 86, 1)"
        ],
        "borderWidth": 1
      }
    ]
  },
  "options": {
    "scales": {
      "y": {
        "beginAtZero": true
      }
    }
  }
}
```
:::
