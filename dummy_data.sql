--
-- PostgreSQL database dump
--

-- Dumped from database version 14.1 (Debian 14.1-1.pgdg110+1)
-- Dumped by pg_dump version 14.1 (Debian 14.1-1.pgdg110+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: attack_type; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.attack_type (id, att_type, attack_radius, attack_damage) FROM stdin;
1	r3d20	3	20
2	r6d5	6	5
3	r10d2	10	2
\.


--
-- Data for Name: levels_fixture; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.levels_fixture (id, start_date, end_date, no_of_bombs) FROM stdin;
1	2022-01-01	2022-02-01	10
\.


--
-- Data for Name: user; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public."user" (id, name, email, phone, username, overall_rating, is_pragyan, password, is_verified) FROM stdin;
1	attacker	attacker@aot.com	1234567890	4774ck3r	1000	f	pass	t
2	defender	defender@aot.com	9876543210	d3f3nd3r	1000	f	pass	t
\.


--
-- Data for Name: map_layout; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.map_layout (id, player, level_id) FROM stdin;
1	2	1
\.


--
-- Data for Name: game; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.game (id, attack_id, defend_id, map_layout_id, attack_score, defend_score) FROM stdin;
1	1	2	1	0	0
\.


--
-- Data for Name: attacker_path; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.attacker_path (id, y_coord, x_coord, is_emp, game_id, emp_type, emp_time) FROM stdin;
7	33	6	t	1	3	100
33	12	11	t	1	2	140
1	39	6	f	1	\N	\N
2	38	6	f	1	\N	\N
3	37	6	f	1	\N	\N
4	36	6	f	1	\N	\N
5	35	6	f	1	\N	\N
6	34	6	f	1	\N	\N
8	32	6	f	1	\N	\N
9	31	6	f	1	\N	\N
10	30	6	f	1	\N	\N
11	29	6	f	1	\N	\N
12	28	6	f	1	\N	\N
13	27	6	f	1	\N	\N
14	26	6	f	1	\N	\N
15	25	6	f	1	\N	\N
16	24	6	f	1	\N	\N
17	23	6	f	1	\N	\N
18	22	6	f	1	\N	\N
19	21	6	f	1	\N	\N
20	20	6	f	1	\N	\N
21	19	6	f	1	\N	\N
22	18	6	f	1	\N	\N
23	17	6	f	1	\N	\N
24	16	6	f	1	\N	\N
25	15	6	f	1	\N	\N
26	14	6	f	1	\N	\N
27	13	6	f	1	\N	\N
28	12	6	f	1	\N	\N
29	12	7	f	1	\N	\N
30	12	8	f	1	\N	\N
31	12	9	f	1	\N	\N
32	12	10	f	1	\N	\N
34	13	11	f	1	\N	\N
35	14	11	f	1	\N	\N
36	15	11	f	1	\N	\N
37	16	11	f	1	\N	\N
38	17	11	f	1	\N	\N
39	18	11	f	1	\N	\N
40	19	11	f	1	\N	\N
41	20	11	f	1	\N	\N
42	21	11	f	1	\N	\N
43	22	11	f	1	\N	\N
44	23	11	f	1	\N	\N
45	24	11	f	1	\N	\N
46	25	11	f	1	\N	\N
47	26	11	f	1	\N	\N
49	27	12	f	1	\N	\N
50	27	13	f	1	\N	\N
51	27	14	f	1	\N	\N
52	27	15	f	1	\N	\N
53	27	16	f	1	\N	\N
54	27	17	f	1	\N	\N
55	26	17	f	1	\N	\N
56	25	17	f	1	\N	\N
57	24	17	f	1	\N	\N
58	23	17	f	1	\N	\N
59	22	17	f	1	\N	\N
61	20	17	f	1	\N	\N
62	19	17	f	1	\N	\N
64	18	18	f	1	\N	\N
65	18	19	f	1	\N	\N
66	18	20	f	1	\N	\N
67	18	21	f	1	\N	\N
69	19	22	f	1	\N	\N
70	20	22	f	1	\N	\N
72	22	22	f	1	\N	\N
73	23	22	f	1	\N	\N
74	24	22	f	1	\N	\N
75	25	22	f	1	\N	\N
76	26	22	f	1	\N	\N
77	27	22	f	1	\N	\N
78	27	23	f	1	\N	\N
79	27	24	f	1	\N	\N
80	27	25	f	1	\N	\N
81	27	26	f	1	\N	\N
82	27	27	f	1	\N	\N
84	26	28	f	1	\N	\N
85	25	28	f	1	\N	\N
86	24	28	f	1	\N	\N
87	23	28	f	1	\N	\N
88	22	28	f	1	\N	\N
89	21	28	f	1	\N	\N
90	20	28	f	1	\N	\N
91	19	28	f	1	\N	\N
92	18	28	f	1	\N	\N
93	17	28	f	1	\N	\N
94	16	28	f	1	\N	\N
95	15	28	f	1	\N	\N
96	14	28	f	1	\N	\N
97	13	28	f	1	\N	\N
99	11	28	f	1	\N	\N
100	10	28	f	1	\N	\N
101	9	28	f	1	\N	\N
102	8	28	f	1	\N	\N
103	7	28	f	1	\N	\N
104	6	28	f	1	\N	\N
105	6	29	f	1	\N	\N
106	6	30	f	1	\N	\N
107	6	31	f	1	\N	\N
108	6	32	f	1	\N	\N
48	27	11	t	1	2	180
60	21	17	t	1	1	220
63	18	17	t	1	1	260
68	18	22	t	1	1	300
71	21	22	t	1	1	340
83	27	28	t	1	2	380
98	12	28	t	1	2	420
109	6	33	t	1	3	460
\.


--
-- Data for Name: block_type; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.block_type (id, name, width, height, entrance_x, entrance_y) FROM stdin;
4	road	1	1	0	0
1	3x3_wt2	3	3	1	0
2	4x4_wt3	4	4	0	2
3	5x5_wt1	5	5	2	0
\.


--
-- Data for Name: building_weights; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.building_weights ("time", building_id, weight) FROM stdin;
9	1	200
10	1	170
11	1	140
12	1	110
13	1	80
14	1	50
15	1	20
16	1	10
9	2	20
10	2	60
11	2	100
12	2	140
13	2	180
14	2	220
15	2	260
16	2	300
9	3	100
10	3	90
11	3	80
12	3	70
13	3	70
14	3	80
15	3	90
16	3	100
\.


--
-- Data for Name: level_constraints; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.level_constraints (level_id, block_id, no_of_buildings) FROM stdin;
1	1	15
1	2	20
1	3	15
\.


--
-- Data for Name: map_spaces; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.map_spaces (id, map_id, blk_type, x_coordinate, y_coordinate, rotation) FROM stdin;
1	1	4	6	1	0
2	1	4	6	2	0
3	1	4	6	3	0
4	1	4	6	4	0
5	1	4	6	5	0
6	1	4	6	6	0
7	1	4	6	7	0
8	1	4	6	8	0
9	1	4	6	9	0
10	1	4	6	10	0
11	1	4	6	11	0
12	1	4	6	12	0
13	1	4	6	13	0
14	1	4	6	14	0
15	1	4	6	15	0
16	1	4	6	16	0
17	1	4	6	17	0
18	1	4	6	18	0
19	1	4	6	19	0
20	1	4	6	20	0
21	1	4	6	21	0
22	1	4	6	22	0
23	1	4	6	23	0
24	1	4	6	24	0
25	1	4	6	25	0
26	1	4	6	26	0
27	1	4	6	27	0
28	1	4	6	28	0
29	1	4	6	29	0
30	1	4	6	30	0
31	1	4	6	31	0
32	1	4	6	32	0
33	1	4	6	33	0
34	1	4	6	34	0
35	1	4	6	35	0
36	1	4	6	36	0
37	1	4	6	37	0
38	1	4	6	38	0
39	1	4	11	1	0
40	1	4	11	2	0
41	1	4	11	3	0
42	1	4	11	4	0
43	1	4	11	5	0
44	1	4	11	6	0
45	1	4	11	7	0
46	1	4	11	8	0
47	1	4	11	9	0
48	1	4	11	10	0
49	1	4	11	11	0
50	1	4	11	12	0
51	1	4	11	13	0
52	1	4	11	14	0
53	1	4	11	15	0
54	1	4	11	16	0
55	1	4	11	17	0
56	1	4	11	18	0
57	1	4	11	19	0
58	1	4	11	20	0
59	1	4	11	21	0
60	1	4	11	22	0
61	1	4	11	23	0
62	1	4	11	24	0
63	1	4	11	25	0
64	1	4	11	26	0
65	1	4	11	27	0
66	1	4	11	28	0
67	1	4	11	29	0
68	1	4	11	30	0
69	1	4	11	31	0
70	1	4	11	32	0
71	1	4	11	33	0
72	1	4	11	34	0
73	1	4	11	35	0
74	1	4	11	36	0
75	1	4	11	37	0
76	1	4	11	38	0
77	1	4	17	1	0
78	1	4	17	2	0
79	1	4	17	3	0
80	1	4	17	4	0
81	1	4	17	5	0
82	1	4	17	6	0
83	1	4	17	7	0
84	1	4	17	8	0
85	1	4	17	9	0
86	1	4	17	10	0
87	1	4	17	11	0
88	1	4	17	12	0
89	1	4	17	13	0
90	1	4	17	14	0
91	1	4	17	15	0
92	1	4	17	16	0
93	1	4	17	17	0
94	1	4	17	18	0
95	1	4	17	19	0
96	1	4	17	20	0
97	1	4	17	21	0
98	1	4	17	22	0
99	1	4	17	23	0
100	1	4	17	24	0
101	1	4	17	25	0
102	1	4	17	26	0
103	1	4	17	27	0
104	1	4	17	28	0
105	1	4	17	29	0
106	1	4	17	30	0
107	1	4	17	31	0
108	1	4	17	32	0
109	1	4	17	33	0
110	1	4	17	34	0
111	1	4	17	35	0
112	1	4	17	36	0
113	1	4	17	37	0
114	1	4	17	38	0
115	1	4	22	1	0
116	1	4	22	2	0
117	1	4	22	3	0
118	1	4	22	4	0
119	1	4	22	5	0
120	1	4	22	6	0
121	1	4	22	7	0
122	1	4	22	8	0
123	1	4	22	9	0
124	1	4	22	10	0
125	1	4	22	11	0
126	1	4	22	12	0
127	1	4	22	13	0
128	1	4	22	14	0
129	1	4	22	15	0
130	1	4	22	16	0
131	1	4	22	17	0
132	1	4	22	18	0
133	1	4	22	19	0
134	1	4	22	20	0
135	1	4	22	21	0
136	1	4	22	22	0
137	1	4	22	23	0
138	1	4	22	24	0
139	1	4	22	25	0
140	1	4	22	26	0
141	1	4	22	27	0
142	1	4	22	28	0
143	1	4	22	29	0
144	1	4	22	30	0
145	1	4	22	31	0
146	1	4	22	32	0
147	1	4	22	33	0
148	1	4	22	34	0
149	1	4	22	35	0
150	1	4	22	36	0
151	1	4	22	37	0
152	1	4	22	38	0
153	1	4	28	1	0
154	1	4	28	2	0
155	1	4	28	3	0
156	1	4	28	4	0
157	1	4	28	5	0
158	1	4	28	6	0
159	1	4	28	7	0
160	1	4	28	8	0
161	1	4	28	9	0
162	1	4	28	10	0
163	1	4	28	11	0
164	1	4	28	12	0
165	1	4	28	13	0
166	1	4	28	14	0
167	1	4	28	15	0
168	1	4	28	16	0
169	1	4	28	17	0
170	1	4	28	18	0
171	1	4	28	19	0
172	1	4	28	20	0
173	1	4	28	21	0
174	1	4	28	22	0
175	1	4	28	23	0
176	1	4	28	24	0
177	1	4	28	25	0
178	1	4	28	26	0
179	1	4	28	27	0
180	1	4	28	28	0
181	1	4	28	29	0
182	1	4	28	30	0
183	1	4	28	31	0
184	1	4	28	32	0
185	1	4	28	33	0
186	1	4	28	34	0
187	1	4	28	35	0
188	1	4	28	36	0
189	1	4	28	37	0
190	1	4	28	38	0
191	1	4	33	1	0
192	1	4	33	2	0
193	1	4	33	3	0
194	1	4	33	4	0
195	1	4	33	5	0
196	1	4	33	6	0
197	1	4	33	7	0
198	1	4	33	8	0
199	1	4	33	9	0
200	1	4	33	10	0
201	1	4	33	11	0
202	1	4	33	12	0
203	1	4	33	13	0
204	1	4	33	14	0
205	1	4	33	15	0
206	1	4	33	16	0
207	1	4	33	17	0
208	1	4	33	18	0
209	1	4	33	19	0
210	1	4	33	20	0
211	1	4	33	21	0
212	1	4	33	22	0
213	1	4	33	23	0
214	1	4	33	24	0
215	1	4	33	25	0
216	1	4	33	26	0
217	1	4	33	27	0
218	1	4	33	28	0
219	1	4	33	29	0
220	1	4	33	30	0
221	1	4	33	31	0
222	1	4	33	32	0
223	1	4	33	33	0
224	1	4	33	34	0
225	1	4	33	35	0
226	1	4	33	36	0
227	1	4	33	37	0
228	1	4	33	38	0
229	1	4	2	6	0
230	1	4	3	6	0
231	1	4	4	6	0
232	1	4	5	6	0
233	1	4	7	6	0
234	1	4	8	6	0
235	1	4	9	6	0
236	1	4	10	6	0
237	1	4	12	6	0
238	1	4	13	6	0
239	1	4	14	6	0
240	1	4	15	6	0
241	1	4	16	6	0
242	1	4	18	6	0
243	1	4	19	6	0
244	1	4	20	6	0
245	1	4	21	6	0
246	1	4	23	6	0
247	1	4	24	6	0
248	1	4	25	6	0
249	1	4	26	6	0
250	1	4	27	6	0
251	1	4	29	6	0
252	1	4	30	6	0
253	1	4	31	6	0
254	1	4	32	6	0
255	1	4	34	6	0
256	1	4	35	6	0
257	1	4	36	6	0
258	1	4	37	6	0
259	1	4	2	12	0
260	1	4	3	12	0
261	1	4	4	12	0
262	1	4	5	12	0
263	1	4	7	12	0
264	1	4	8	12	0
265	1	4	9	12	0
266	1	4	10	12	0
267	1	4	12	12	0
268	1	4	13	12	0
269	1	4	14	12	0
270	1	4	15	12	0
271	1	4	16	12	0
272	1	4	18	12	0
273	1	4	19	12	0
274	1	4	20	12	0
275	1	4	21	12	0
276	1	4	23	12	0
277	1	4	24	12	0
278	1	4	25	12	0
279	1	4	26	12	0
280	1	4	27	12	0
281	1	4	29	12	0
282	1	4	30	12	0
283	1	4	31	12	0
284	1	4	32	12	0
285	1	4	34	12	0
286	1	4	35	12	0
287	1	4	36	12	0
288	1	4	37	12	0
289	1	4	2	18	0
290	1	4	3	18	0
291	1	4	4	18	0
292	1	4	5	18	0
293	1	4	7	18	0
294	1	4	8	18	0
295	1	4	9	18	0
296	1	4	10	18	0
297	1	4	12	18	0
298	1	4	13	18	0
299	1	4	14	18	0
300	1	4	15	18	0
301	1	4	16	18	0
302	1	4	18	18	0
303	1	4	19	18	0
304	1	4	20	18	0
305	1	4	21	18	0
306	1	4	23	18	0
307	1	4	24	18	0
308	1	4	25	18	0
309	1	4	26	18	0
310	1	4	27	18	0
311	1	4	29	18	0
312	1	4	30	18	0
313	1	4	31	18	0
314	1	4	32	18	0
315	1	4	34	18	0
316	1	4	35	18	0
317	1	4	36	18	0
318	1	4	37	18	0
319	1	4	2	21	0
320	1	4	3	21	0
321	1	4	4	21	0
322	1	4	5	21	0
323	1	4	7	21	0
324	1	4	8	21	0
325	1	4	9	21	0
326	1	4	10	21	0
327	1	4	12	21	0
328	1	4	13	21	0
329	1	4	14	21	0
330	1	4	15	21	0
331	1	4	16	21	0
332	1	4	18	21	0
333	1	4	19	21	0
334	1	4	20	21	0
335	1	4	21	21	0
336	1	4	23	21	0
337	1	4	24	21	0
338	1	4	25	21	0
339	1	4	26	21	0
340	1	4	27	21	0
341	1	4	29	21	0
342	1	4	30	21	0
343	1	4	31	21	0
344	1	4	32	21	0
345	1	4	34	21	0
346	1	4	35	21	0
347	1	4	36	21	0
348	1	4	37	21	0
349	1	4	2	27	0
350	1	4	3	27	0
351	1	4	4	27	0
352	1	4	5	27	0
353	1	4	7	27	0
354	1	4	8	27	0
355	1	4	9	27	0
356	1	4	10	27	0
357	1	4	12	27	0
358	1	4	13	27	0
359	1	4	14	27	0
360	1	4	15	27	0
361	1	4	16	27	0
362	1	4	18	27	0
363	1	4	19	27	0
364	1	4	20	27	0
365	1	4	21	27	0
366	1	4	23	27	0
367	1	4	24	27	0
368	1	4	25	27	0
369	1	4	26	27	0
370	1	4	27	27	0
371	1	4	29	27	0
372	1	4	30	27	0
373	1	4	31	27	0
374	1	4	32	27	0
375	1	4	34	27	0
376	1	4	35	27	0
377	1	4	36	27	0
378	1	4	37	27	0
379	1	4	2	33	0
380	1	4	3	33	0
381	1	4	4	33	0
382	1	4	5	33	0
383	1	4	7	33	0
384	1	4	8	33	0
385	1	4	9	33	0
386	1	4	10	33	0
387	1	4	12	33	0
388	1	4	13	33	0
389	1	4	14	33	0
390	1	4	15	33	0
391	1	4	16	33	0
392	1	4	18	33	0
393	1	4	19	33	0
394	1	4	20	33	0
395	1	4	21	33	0
396	1	4	23	33	0
397	1	4	24	33	0
398	1	4	25	33	0
399	1	4	26	33	0
400	1	4	27	33	0
401	1	4	29	33	0
402	1	4	30	33	0
403	1	4	31	33	0
404	1	4	32	33	0
405	1	4	34	33	0
406	1	4	35	33	0
407	1	4	36	33	0
408	1	4	37	33	0
409	1	1	3	3	90
410	1	1	3	9	90
411	1	1	3	15	90
412	1	1	3	22	90
413	1	1	3	28	90
414	1	1	3	34	90
415	1	1	34	3	270
416	1	1	34	9	270
417	1	1	34	15	270
418	1	1	34	22	270
419	1	1	34	28	270
420	1	1	34	34	270
421	1	2	7	2	0
422	1	2	7	8	0
423	1	2	7	14	0
424	1	2	7	22	0
425	1	2	7	28	0
426	1	2	7	34	0
427	1	2	18	2	270
428	1	2	18	8	270
429	1	2	18	14	270
430	1	2	18	22	270
431	1	2	18	28	270
432	1	2	18	34	270
433	1	2	29	2	180
434	1	2	29	8	180
435	1	2	29	14	180
436	1	2	29	22	180
437	1	2	29	28	180
438	1	2	29	34	180
439	1	3	12	1	180
440	1	3	12	7	180
441	1	3	12	13	180
442	1	3	12	22	0
443	1	3	12	28	0
444	1	3	12	34	0
445	1	3	23	1	180
446	1	3	23	7	180
447	1	3	23	13	180
448	1	3	23	22	0
449	1	3	23	28	0
450	1	3	23	34	0
451	1	4	6	39	0
\.


--
-- Data for Name: shortest_path; Type: TABLE DATA; Schema: public; Owner: aot
--

COPY public.shortest_path (base_id, source_x, source_y, dest_x, dest_y, pathlist) FROM stdin;
\.


--
-- Name: attack_type_att_type_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.attack_type_att_type_seq', 1, false);


--
-- Name: attack_type_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.attack_type_id_seq', 1, false);


--
-- Name: attacker_path_x_coord_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.attacker_path_x_coord_seq', 1, false);


--
-- Name: attacker_path_y_coord_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.attacker_path_y_coord_seq', 1, false);


--
-- Name: block_type_height_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.block_type_height_seq', 1, false);


--
-- Name: block_type_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.block_type_id_seq', 1, false);


--
-- Name: block_type_width_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.block_type_width_seq', 1, false);


--
-- Name: game_attack_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.game_attack_id_seq', 1, false);


--
-- Name: game_defend_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.game_defend_id_seq', 1, false);


--
-- Name: game_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.game_id_seq', 1, false);


--
-- Name: game_map_layout_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.game_map_layout_id_seq', 1, false);


--
-- Name: map_layout_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_layout_id_seq', 1, false);


--
-- Name: map_layout_level_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_layout_level_id_seq', 1, false);


--
-- Name: map_layout_player_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_layout_player_seq', 1, false);


--
-- Name: map_spaces_blk_type_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_spaces_blk_type_seq', 1, false);


--
-- Name: map_spaces_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_spaces_id_seq', 1, false);


--
-- Name: map_spaces_map_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.map_spaces_map_id_seq', 1, false);


--
-- Name: user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: aot
--

SELECT pg_catalog.setval('public.user_id_seq', 1, false);


--
-- PostgreSQL database dump complete
--
