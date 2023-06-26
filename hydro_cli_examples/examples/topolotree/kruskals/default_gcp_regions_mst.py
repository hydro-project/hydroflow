import folium
import pandas as pd


# Create the graph from the provided regions, latitudes, and longitudes
regions = ["Iowa", "Oregon", "Northern Virginia", "South Carolina", "Belgium", "London", "Frankfurt",
           "Singapore", "Taiwan", "Tokyo", "Sydney", "Sao Paulo", "Mumbai", "Montreal", "Netherlands",
           "Los Angeles", "Salt Lake City", "Las Vegas", "Zurich", "Finland", "Jakarta", "Hong Kong",
           "Osaka", "Seoul"]
latitudes = [41.8780, 44.4268, 38.9072, 33.8361, 50.5039, 51.5074, 50.1109, 1.3521, 25.0330, 35.6895,
             -33.8688, -23.5505, 19.0760, 45.5017, 52.3702, 34.0522, 40.7608, 36.1699, 47.3769, 60.1699,
             -6.2088, 22.3193, 34.6937, 37.5665]
longitudes = [-93.0977, -121.9707, -77.0369, -80.9450, 4.4699, -0.1278, 8.6821, 103.8198, 121.5654,
              139.6917, 151.2093, -46.6333, 72.8777, -73.5673, 4.8952, -118.2437, -111.8910, -115.1398,
              8.5417, 24.9384, 106.8456, 114.1694, 135.5022, 126.9779]


class UnionFind:
    def __init__(self, n):
        self.parent = list(range(n))
        self.rank = [0] * n

    def find(self, x):
        if self.parent[x] != x:
            self.parent[x] = self.find(self.parent[x])
        return self.parent[x]

    def union(self, x, y):
        root_x = self.find(x)
        root_y = self.find(y)

        if root_x != root_y:
            if self.rank[root_x] < self.rank[root_y]:
                self.parent[root_x] = root_y
            elif self.rank[root_x] > self.rank[root_y]:
                self.parent[root_y] = root_x
            else:
                self.parent[root_y] = root_x
                self.rank[root_x] += 1


def kruskals_algorithm(graph):
    edges = []
    for u in range(len(graph)):
        for v in range(u + 1, len(graph)):
            if graph[u][v] != 0:
                edges.append((u, v, graph[u][v]))

    edges.sort(key=lambda x: x[2])  # Sort edges by weight

    minimum_spanning_tree = []
    uf = UnionFind(len(graph))

    for u, v, weight in edges:
        if uf.find(u) != uf.find(v):  # Check if adding the edge forms a cycle
            uf.union(u, v)
            minimum_spanning_tree.append((u, v, weight))

    return minimum_spanning_tree


def visualize_graph(graph, minimum_spanning_tree):
    # Create a Folium map centered on the first vertex
    map_graph = folium.Map(location=[graph[0][0], graph[0][1]], zoom_start=2)

    # Add vertices to the map
    for vertex in graph:
        folium.CircleMarker(
            location=[vertex[0], vertex[1]],
            radius=3,
            color='red',
            fill=True,
            fill_color='red',
            # popup=f"Region: {vertex[2]}",
            fill_opacity=1,
            tooltip=vertex[2]
            # icon=folium.Icon(color="blue")
        ).add_to(map_graph)

    # Add edges in the minimum spanning tree to the map
    for u, v, weight in minimum_spanning_tree:
        folium.PolyLine(
            locations=[[graph[u][0], graph[u][1]], [graph[v][0], graph[v][1]]],
            color="black",
            weight=1,
            opacity=1,
            popup=f"Weight: {weight}"
        ).add_to(map_graph)

    return map_graph


def create_graph():
        
        df = pd.read_csv('gcp-cross-region-latencies.csv')

        outer = []
        for i in range(len(regions)):
            inner = []
            for j in range(len(regions)):
                filter1 = df['From'] == regions[i]
                new_df = df.where(filter1).dropna()
                new_df = new_df[regions[j]]
                edge_weight = new_df.iloc[0]
                inner.append(edge_weight)
            outer.append(inner)
        
        return outer


graph = []
for i in range(len(regions)):
    graph.append([latitudes[i], longitudes[i], regions[i]])


# Example graph represented as an adjacency matrix
adjacency_matrix = create_graph()


minimum_spanning_tree = kruskals_algorithm(adjacency_matrix)

# Visualize the graph and minimum spanning tree using Folium
map_graph = visualize_graph(graph, minimum_spanning_tree)
map_graph.save("graph.html")
