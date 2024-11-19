import React from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

import styles from './index.module.css';

export default function Home() {
  return (
    <Layout>
      <main>
        <div className={styles["jumbo"]}>
          <img src="/img/hydro-logo.svg" alt="Hydro Logo" style={{
            width: "650px",
            marginLeft: "auto",
            marginRight: "auto",
          }} />
          <h2 className={styles["indexTitle"]}>build for <span className={styles["titleEveryScale"]}>every scale</span></h2>

          <div style={{ marginTop: "20px" }}>
            <p className={styles["blurb"]}>The Hydro Project at UC Berkeley is developing <b>cloud-native</b> programming models that allow <b>anyone</b> to develop <b>scalable and resilient distributed applications</b>. Our research spans across <b>databases, distributed systems, and programming languages</b> to deliver a modern, end-to-end stack for cloud programming.</p>
            <div style={{
              display: "flex",
              flexDirection: "row",
              marginTop: "30px",
              marginBottom: "30px",
              justifyContent: "center",
              flexWrap: "wrap"
            }}>
              <Link to="/docs/hydroflow_plus/quickstart/" className="button button--primary button--lg" style={{
                margin: "10px",
                marginTop: 0,
                fontSize: "1.4em",
                color: "white"
              }}>Get Started</Link>

              <Link to="/research/" className="button button--outline button--secondary button--lg" style={{
                margin: "10px",
                marginTop: 0,
                fontSize: "1.4em"
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
