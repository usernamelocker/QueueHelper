import { useEffect, useState } from "react";

export interface Champion {
  id: string;
  key: string;
  name: string;
}

const DD_VERSION = "16.12.1";
const CACHE_KEY = `queue-helper-champions-${DD_VERSION}`;

export function useChampions() {
  const [champions, setChampions] = useState<Champion[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const cached = localStorage.getItem(CACHE_KEY);
    if (cached) {
      try {
        setChampions(JSON.parse(cached));
        setLoading(false);
        return;
      } catch {
        localStorage.removeItem(CACHE_KEY);
      }
    }

    const url = `https://ddragon.leagueoflegends.com/cdn/${DD_VERSION}/data/en_US/champion.json`;

    fetch(url)
      .then((res) => res.json())
      .then((data) => {
        const list: Champion[] = Object.values(data.data).map(
          (c: any) => ({
            id: c.id,
            key: c.key,
            name: c.name,
          })
        );
        list.sort((a, b) => a.name.localeCompare(b.name));
        localStorage.setItem(CACHE_KEY, JSON.stringify(list));
        setChampions(list);
        setLoading(false);
      })
      .catch(() => {
        setLoading(false);
      });
  }, []);

  return { champions, loading };
}
