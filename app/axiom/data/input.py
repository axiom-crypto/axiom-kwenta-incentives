import json


def generate_json():
    data = {
        "blockNumbers": [5147955 for i in range(40)],
        "txIdxs": [31 for i in range(40)],
        "logIdxs": [0 for i in range(40)],
        "referrerId": 1,
        "numClaims": 1,
    }

    json_data = json.dumps(data, indent=4)
    print(json_data)


if __name__ == "__main__":
    generate_json()
