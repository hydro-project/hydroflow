import React from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

import styles from './index.module.css';

export default function Home() {
  return (
    <Layout>
      <main>
        <div className={styles["jumbo"]}>
          <h2 className={styles["indexTitle"]}>Build Software for <span className={styles["titleEveryScale"]}>Every Scale</span></h2>

          <div style={{ marginTop: "20px" }}>
            <p className={styles["blurb"]}>The Hydro Project at UC Berkeley is developing <b>cloud-native</b> programming models that allow <b>anyone</b> to develop <b>scalable and resilient distributed applications</b> that take full advantage of cloud elasticity. Our research spans across <b>databases, distributed systems, and programming languages</b> to deliver a modern, end-to-end stack for cloud programming.</p>
            <div style={{
              display: "flex",
              flexDirection: "row",
              marginTop: "30px",
              marginBottom: "30px",
              justifyContent: "center",
              flexWrap: "wrap"
            }}>
              <Link to="/docs/hydroflow/quickstart/" style={{
                display: "block",
                padding: "15px",
                textDecoration: "none",
                color: "white",
                textAlign: "center",
                background: "linear-gradient(86.9deg, #3CB6FB -8.29%, #2DD3EA 109.09%)",
                boxShadow: "0px 3px 8px rgba(0, 0, 0, 0.25)",
                borderRadius: "15px",
                width: "200px",
                margin: "10px",
                marginTop: 0,
                fontSize: "1.25em"
              }}>Get Started</Link>

              <Link to="/research/" style={{
                display: "block",
                padding: "15px",
                textDecoration: "none",
                color: "black",
                textAlign: "center",
                background: "white",
                boxShadow: "0px 3px 8px rgba(0, 0, 0, 0.25)",
                borderRadius: "15px",
                width: "200px",
                margin: "10px",
                marginTop: 0,
                fontSize: "1.25em"
              }}>
                Latest Research
              </Link>
            </div>
          </div>
        </div>
      </main>
    </Layout>
  );
}
