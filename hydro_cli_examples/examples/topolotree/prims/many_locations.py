import random
from global_land_mask import globe
from math import radians, cos, sin, asin, sqrt

gcp_regions = ["Iowa", "Oregon", "Northern Virginia", "South Carolina", "Belgium", "London", "Frankfurt", "Singapore", "Taiwan", "Tokyo",
               "Sydney", "Sao Paulo", "Mumbai", "Montreal", "Netherlands", "Los Angeles", "Salt Lake City", "Las Vegas", "Zurich", "Finland",
               "Jakarta", "Hong Kong", "Osaka", "Seoul"]

latitudes = [41.8780, 44.4268, 38.9072, 33.8361, 50.5039, 51.5074, 50.1109,
                     1.3521, 25.0330, 35.6895, -33.8688, -23.5505, 19.0760, 45.5017,
                     52.3702, 34.0522, 40.7608, 36.1699, 47.3769, 60.1699, -6.2088,
                     22.3193, 34.6937, 37.5665]
longitudes = [-93.0977, -121.9707, -77.0369, -80.9450, 4.4699, -0.1278, 8.6821,
                      103.8198, 121.5654, 139.6917, 151.2093, -46.6333, 72.8777, -73.5673,
                      4.8952, -118.2437, -111.8910, -115.1398, 8.5417, 24.9384, 106.8456,
                      114.1694, 135.5022, 126.9779]


def generate_random_land_coordinates(num_coords):
    coords = []

    while num_coords > 0:
        latitude = random.uniform(-90, 90)
        longitude = random.uniform(-180, 180)

        if globe.is_land(latitude, longitude):
            num_coords -= 1
            coords.append((latitude, longitude))
    
    return coords


def distance_between_coordinates(coord1, coord2):

    long1 = radians(coord1[1])
    long2 = radians(coord2[1])
    lat1 = radians(coord1[0])
    lat2 = radians(coord2[0])

    dlong = long2 - long1
    dlat = lat2 - lat1
    a = sin(dlat / 2)**2 + cos(lat1) * cos(lat2) * sin(dlong / 2)**2
    c = 2 * asin(sqrt(a))
    
    return c * 6371


def min_distance_location(coord):

    min_distance = 10 ** 20
    location = None

    for i in range(len(latitudes)):

        gcp_coord = (latitudes[i], longitudes[i])
        cur_dist = distance_between_coordinates(coord, gcp_coord)
        if cur_dist < min_distance:
            min_distance = cur_dist
            location = gcp_regions[i]
    
    return location

