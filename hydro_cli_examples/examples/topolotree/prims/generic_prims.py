import sys
import pandas as pd
import networkx as nx
import matplotlib.pyplot as plt
import folium
 
 
class Graph():
    def __init__(self, vertices):
        self.V = vertices
        self.graph = self.create_graph(vertices)
    

    # Helper function to create the graph
    def create_graph(self, vertices):
        gcp_regions = ["Iowa", "Oregon", "Northern Virginia", "South Carolina", "Belgium", "London", "Frankfurt", "Singapore", "Taiwan", "Tokyo",
               "Sydney", "Sao Paulo", "Mumbai", "Montreal", "Netherlands", "Los Angeles", "Salt Lake City", "Las Vegas", "Zurich", "Finland",
               "Jakarta", "Hong Kong", "Osaka", "Seoul"]
        
        df = pd.read_csv('gcp-cross-region-latencies.csv')

        outer = []
        for i in range(vertices):
            inner = []
            for j in range(vertices):
                filter1 = df['From'] == gcp_regions[i]
                new_df = df.where(filter1).dropna()
                new_df = new_df[gcp_regions[j]]
                edge_weight = new_df.iloc[0]
                inner.append(edge_weight)
            outer.append(inner)
        
        return outer
    

    def printMST(self, parent):
        G = nx.Graph()

        for i in range(1, self.V):
            G.add_edge(parent[i], i)

        # Create a Folium map centered on a specific location
        m = folium.Map(location=[0, 0], zoom_start=2)

        # Get the latitude and longitude of the nodes
        latitudes = [41.8780, 44.4268, 38.9072, 33.8361, 50.5039, 51.5074, 50.1109,
                     1.3521, 25.0330, 35.6895, -33.8688, -23.5505, 19.0760, 45.5017,
                     52.3702, 34.0522, 40.7608, 36.1699, 47.3769, 60.1699, -6.2088,
                     22.3193, 34.6937, 37.5665]
        longitudes = [-93.0977, -121.9707, -77.0369, -80.9450, 4.4699, -0.1278, 8.6821,
                      103.8198, 121.5654, 139.6917, 151.2093, -46.6333, 72.8777, -73.5673,
                      4.8952, -118.2437, -111.8910, -115.1398, 8.5417, 24.9384, 106.8456,
                      114.1694, 135.5022, 126.9779]

        # Add the nodes to the map
        for lat, lon in zip(latitudes, longitudes):
            folium.CircleMarker(
                location=[lat, lon],
                radius=5,
                color='red',
                fill=True,
                fill_color='red',
                fill_opacity=1
            ).add_to(m)

        # Add the edges to the map
        for u, v in G.edges():
            folium.PolyLine(
                locations=[[latitudes[u], longitudes[u]], [latitudes[v], longitudes[v]]],
                color='black',
                weight=1,
                opacity=1
            ).add_to(m)

        # Display the map
        m.save('mst_map.html')


    def minKey(self, key, mstSet):
 
        min = sys.maxsize 
 
        for v in range(self.V):
            if key[v] < min and mstSet[v] == False:
                min = key[v]
                min_index = v
 
        return min_index
 
    
    def primMST(self):
 
        # Key values used to pick minimum weight edge in cut
        key = [sys.maxsize] * self.V
        parent = [None] * self.V 
        # Make key 0 so that this vertex is picked as first vertex
        key[0] = 0
        mstSet = [False] * self.V
 
        parent[0] = -1 
 
        for _ in range(self.V):
 
            u = self.minKey(key, mstSet)
            mstSet[u] = True
 
            for v in range(self.V):
 
                if self.graph[u][v] > 0 and mstSet[v] == False \
                and key[v] > self.graph[u][v]:
                    key[v] = self.graph[u][v]
                    parent[v] = u
 
        self.printMST(parent)
 
 
if __name__ == '__main__':
    g = Graph(24)
    g.primMST()